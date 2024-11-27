// Compare bytes: __m128i _mm_cmpeq_epi8 (__m128i a, __m128i b)
// https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#ssetechs=SSE,SSE2,SSE3,SSSE3,SSE4_1,SSE4_2&ig_expand=919,921,922,920,876&text=cmp
//All zero: int _mm_test_all_zeros (__m128i mask, __m128i a)
// __int64 _mm_extract_epi64


macro_rules! print_reg {
    ( $x:expr, $y:expr ) => {
        {
           let low = _mm_extract_epi64::<0>($y);
        let high = _mm_extract_epi64::<1>($y);
        println!("{} {:016x} {:016x}", $x, high, low);
        }
    };

}

const NEWLINE:u8 = b'\n';
const RETURN_CARRIAGE:u8 = b'\r'; // Windows sucks
const COMMA:u8 = b',';

pub fn search_char(haystack: &[u8]) -> Option<usize>{

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse4.1") {
            return unsafe { search_simd_2(COMMA, NEWLINE, haystack) };
        }
    }

    search_basic_2(COMMA, NEWLINE, haystack);
    unreachable!("TODO: fallback");
}

#[target_feature(enable = "sse4.1")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_simd_2(needle1 :u8, needle2: u8, mut haystack: &[u8]) -> Option<usize> {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut offset:usize = 0;
    let simd_a = _mm_set1_epi8(needle1 as i8);
    let simd_b = _mm_set1_epi8(needle2 as i8);

    //print_reg!("Comma: ", simd_a);
    //print_reg!("Newline: ", simd_b);

    while haystack.len() >= 16 {
        //println!("Loop 1");
        let haystack_reg = _mm_loadu_si128(haystack.as_ptr() as *const _); // SEE2
        //let haystack_reg = _mm_load_si128(src.as_ptr() as *const _); // Load aligned

        //print_reg!("Haystack", haystack_reg);

        let has_comma = _mm_cmpeq_epi8(haystack_reg, simd_a); // SEE2
        let has_newline = _mm_cmpeq_epi8(haystack_reg, simd_b); // SEE2
        let has_comma_or_newline = _mm_or_si128(has_comma, has_newline); // SEE2

        //print_reg!("Match", has_comma_or_newline);
        
        let low = _mm_extract_epi64::<0>(has_comma_or_newline);
        let high = _mm_extract_epi64::<1>(has_comma_or_newline);
        //println!("{:x} {:x}", high, low);


        let match_summary = _mm_movemask_epi8(has_comma_or_newline);
        //println!("Match summary: {:b}", match_summary);
        let first_match_idx = match_summary.trailing_zeros();
        //println!("Idx: {}", first_match_idx);

        if first_match_idx < i32::BITS {
            return Some(offset + usize::try_from(first_match_idx).unwrap());
        }
        
        haystack = &haystack[16..];
        offset += 16;
    }

    search_basic_2(needle1, needle2, haystack).map(|v| v+offset)
}

fn search_basic_2(needle1 :u8, needle2: u8, haystack: &[u8]) -> Option<usize>{
    let mut i = 0;
    while i<haystack.len(){
        if haystack[i] == needle1 || haystack[i] == needle2 {
            return Some(i) 
        }
        i += 1;
    }
    None


}

mod tests {
    use super::*;

    #[test]
    fn search_char_test() {
        let data = "00000000,00000000";
        assert_eq!(search_char(data.as_bytes()), Some(8));
    }
    
    #[test]
    fn search_char_test_2() {
        let data = "00000000000000000,";
        assert_eq!(search_char(data.as_bytes()), Some(17));
    }
    
    #[test]
    fn search_char_test_3() {
        let data = "00000,";
        assert_eq!(search_char(data.as_bytes()), Some(5));
    }
}