pub fn filter_none(row: &[u8], row_bytes: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(row_bytes + 1);
    out.push(0);
    out.extend_from_slice(row);
    out
}

pub fn filter_sub(row: &[u8], row_bytes: usize, bpp: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(row_bytes + 1);
    out.push(1);
    for i in 0..row_bytes {
        let left = if i >= bpp { row[i - bpp] } else { 0 };
        out.push(row[i].wrapping_sub(left));
    }
    out
}

pub fn filter_up(row: &[u8], prev_row: Option<&[u8]>, row_bytes: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(row_bytes + 1);
    out.push(2);
    for i in 0..row_bytes {
        let up = if let Some(pr) = prev_row { pr[i] } else { 0 };
        out.push(row[i].wrapping_sub(up));
    }
    out
}

pub fn filter_average(
    row: &[u8],
    prev_row: Option<&[u8]>,
    row_bytes: usize,
    bpp: usize,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(row_bytes + 1);
    out.push(3);
    for i in 0..row_bytes {
        let left = if i >= bpp { row[i - bpp] } else { 0 };
        let up = if let Some(pr) = prev_row { pr[i] } else { 0 };
        let avg = ((left as u16 + up as u16) / 2) as u8;
        out.push(row[i].wrapping_sub(avg));
    }
    out
}

pub fn filter_paeth(row: &[u8], prev_row: Option<&[u8]>, row_bytes: usize, bpp: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(row_bytes + 1);
    out.push(4);
    for i in 0..row_bytes {
        let a = if i >= bpp { row[i - bpp] } else { 0 }; // left
        let b = if let Some(pr) = prev_row { pr[i] } else { 0 }; // up
        let c = if i >= bpp {
            if let Some(pr) = prev_row {
                pr[i - bpp]
            } else {
                0
            }
        } else {
            0
        }; // upper left

        let p = a as i32 + b as i32 - c as i32;
        let pa = (p - a as i32).abs();
        let pb = (p - b as i32).abs();
        let pc = (p - c as i32).abs();

        let pr = if pa <= pb && pa <= pc {
            a
        } else if pb <= pc {
            b
        } else {
            c
        };

        out.push(row[i].wrapping_sub(pr));
    }
    out
}

/// Apply a specific filter to the entire image data.
pub fn apply_filter(data: &[u8], width: u32, height: u32, bpp: usize, filter_type: u8) -> Vec<u8> {
    let row_bytes = (width as usize) * bpp;
    let mut out = Vec::with_capacity((row_bytes + 1) * (height as usize));

    for y in 0..(height as usize) {
        let start = y * row_bytes;
        let end = start + row_bytes;
        let row = &data[start..end];
        let prev_row = if y > 0 {
            Some(&data[(y - 1) * row_bytes..y * row_bytes])
        } else {
            None
        };

        let filtered_row = match filter_type {
            0 => filter_none(row, row_bytes),
            1 => filter_sub(row, row_bytes, bpp),
            2 => filter_up(row, prev_row, row_bytes),
            3 => filter_average(row, prev_row, row_bytes, bpp),
            4 => filter_paeth(row, prev_row, row_bytes, bpp),
            _ => filter_none(row, row_bytes),
        };
        out.extend_from_slice(&filtered_row);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_none() {
        let row = vec![10, 20, 30, 40];
        let result = filter_none(&row, 4);
        assert_eq!(result, vec![0, 10, 20, 30, 40]);
    }

    #[test]
    fn test_filter_sub() {
        let row = vec![10, 20, 30, 40];
        let result = filter_sub(&row, 4, 2); // bpp = 2
        // Filter sub formula: row[i] - left
        // left: 0, 0, 10, 20
        // Expected:
        // 10 - 0 = 10
        // 20 - 0 = 20
        // 30 - 10 = 20
        // 40 - 20 = 20
        assert_eq!(result, vec![1, 10, 20, 20, 20]);
    }

    #[test]
    fn test_filter_up() {
        let row = vec![10, 20, 30, 40];
        let prev_row = vec![5, 10, 15, 20];
        let result = filter_up(&row, Some(&prev_row), 4);
        assert_eq!(result, vec![2, 5, 10, 15, 20]);
    }

    #[test]
    fn test_apply_filter() {
        let data = vec![
            10, 20, 30, 40,
            50, 60, 70, 80,
        ];
        // Test filter 0 (None)
        let filtered = apply_filter(&data, 2, 2, 2, 0); // width=2, height=2, bpp=2
        assert_eq!(filtered, vec![
            0, 10, 20, 30, 40,
            0, 50, 60, 70, 80
        ]);
    }
}

