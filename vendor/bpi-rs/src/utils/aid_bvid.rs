const XOR_CODE: i64 = 23_442_827_791_579;
const MAX_CODE: i64 = 2_251_799_813_685_247;
const CHARTS: &str = "FcwAPNKTMug3GV5Lj7EJnHpWsx4tb8haYeviqBz6rkCy12mUSDQX9RdoZf";
const PAUL_NUM: i64 = 58;

fn swap_string(s: &str, x: usize, y: usize) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    chars.swap(x, y);
    chars.into_iter().collect()
}

pub fn bvid_to_avid(bvid: &str) -> i64 {
    let s = swap_string(&swap_string(bvid, 3, 9), 4, 7);
    let bv1: Vec<char> = s.chars().skip(3).collect();

    let mut temp: i64 = 0;
    for c in bv1 {
        let idx = CHARTS.find(c).unwrap() as i64;
        temp = temp * PAUL_NUM + idx;
    }

    (temp & MAX_CODE) ^ XOR_CODE
}

pub fn avid_to_bvid(avid: i64) -> String {
    let mut arr = vec!["B".to_string(), "V".to_string(), "1".to_string()];
    arr.resize(12, String::new());
    let mut bv_idx = arr.len() - 1;

    let mut temp = (avid | (MAX_CODE + 1)) ^ XOR_CODE;
    while temp > 0 {
        let idx = (temp % PAUL_NUM) as usize;
        arr[bv_idx] = CHARTS.chars().nth(idx).unwrap().to_string();
        temp /= PAUL_NUM;
        if bv_idx == 0 {
            break;
        }
        bv_idx -= 1;
    }

    let raw = arr.join("");
    swap_string(&swap_string(&raw, 3, 9), 4, 7)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bvid_to_avid() {
        assert_eq!(bvid_to_avid("BV1bx411c7ux"), 10000);
        assert_eq!(bvid_to_avid("BV1bx411c7us"), 10001);
    }

    #[test]
    fn test_avid_to_bvid() {
        assert_eq!(avid_to_bvid(10000), "BV1bx411c7ux");
        assert_eq!(avid_to_bvid(10001), "BV1bx411c7us");
    }
}
