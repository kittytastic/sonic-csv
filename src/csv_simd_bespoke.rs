use memchr::memchr3;
// Compare bytes: __m128i _mm_cmpeq_epi8 (__m128i a, __m128i b)
// https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html#ssetechs=SSE,SSE2,SSE3,SSSE3,SSE4_1,SSE4_2&ig_expand=919,921,922,920,876&text=cmp
//All zero: int _mm_test_all_zeros (__m128i mask, __m128i a)
// __int64 _mm_extract_epi64

#[derive(Debug, Clone)]
pub struct CsvSimdCursor<'a> {
    file_bytes: &'a[u8],
    cursor_pos: usize,
    eol: bool,
    next_val_idx: usize,
    line_num: usize,
}

const NEWLINE:u8 = b'\n';
const RETURN_CARRIAGE:u8 = b'\r'; // Windows sucks
const COMMA:u8 = b',';

impl<'a> CsvSimdCursor<'a>{
    pub fn new(file_bytes: &[u8])->CsvSimdCursor<'_>{
        CsvSimdCursor {
            file_bytes: file_bytes,
            cursor_pos: 0,
            eol: false,
            next_val_idx: 0,
            line_num: 0,
        }
    }

    pub fn next_value(&mut self)->Option<&'a[u8]>{
        if self.eol {return None}
        let todo_lazy = &self.file_bytes[self.cursor_pos..];
        let next_point = memchr3(COMMA, NEWLINE, RETURN_CARRIAGE, &todo_lazy);
        if let Some(val) = next_point{
        let current_byte = todo_lazy[val];
        let new_pos = self.cursor_pos + val;
        if current_byte == COMMA {
            let value_bytes = &self.file_bytes[self.cursor_pos..new_pos];
            self.cursor_pos = new_pos+1; // skip the comma
            self.next_val_idx += 1;
            return Some(value_bytes)
        }
        
        if current_byte == RETURN_CARRIAGE {
            let value_bytes = &self.file_bytes[self.cursor_pos..new_pos];
            self.cursor_pos = new_pos+2; // skip \r\n
            self.eol = true; 
            self.next_val_idx += 1;
            return Some(value_bytes)
        }
        
        if current_byte == NEWLINE {
            let value_bytes = &self.file_bytes[self.cursor_pos..new_pos];
            self.cursor_pos = new_pos+1; // skip \n
            self.next_val_idx += 1;
            self.eol = true; 
            return Some(value_bytes)
        }
        unreachable!();
    }else{
        let new_pos = self.file_bytes.len();
        // Hit EOF, but we didn't start there so there must be bytes to return
        let value_bytes = &self.file_bytes[self.cursor_pos..new_pos];
        self.cursor_pos = new_pos;
        self.next_val_idx += 1;
        self.eol = true;
        Some(value_bytes)
    }
    }

    pub fn get_value(&mut self, idx: usize)->Option<&'a[u8]>{
        if idx<self.next_val_idx{return None}
        let mut r = None;
        for _ in 0..((idx-self.next_val_idx)+1){
            r = self.next_value();
            if r.is_none(){break;}
        }
        r
    }

    pub fn advance_line(&mut self)->bool{
        if self.eol {
            if self.cursor_pos != self.file_bytes.len(){
                self.eol = false;
                self.next_val_idx = 0;
                self.line_num += 1;
                 return true }
            else {return false;}
        }
        
        let mut new_pos = self.cursor_pos;
        while new_pos< self.file_bytes.len(){
            
            if self.file_bytes[new_pos]==RETURN_CARRIAGE {
                self.cursor_pos = new_pos+2; // skip \r\n 
                self.eol = false;
                self.next_val_idx = 0;
                self.line_num += 1;
                return true;
            }
            if self.file_bytes[new_pos]==NEWLINE {
                self.cursor_pos = new_pos+1; // skip \n 
                self.eol = false;
                self.next_val_idx = 0;
                self.line_num += 1;
                return true;
            }
            new_pos += 1;
        }

        self.eol = true;
        self.cursor_pos = new_pos; // hit EOF so move cursor there
        false
    }


    pub fn advance_by_lines(& mut self, lines: usize)->bool{
        let mut still_lines = true;
        for _ in 0..lines{
            still_lines &= self.advance_line();
        }
        return still_lines;
    }

    pub fn at_end(&self)->bool{
        self.cursor_pos == self.file_bytes.len()
    }

    pub fn get_line_number(&self)->usize{
        self.line_num
    }

}



mod tests {
    use super::*;

    #[test]
    fn next_val() {
        let file = "hi,hello\nlater,bye";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.next_value(), Some("hi".as_bytes()));
        assert_eq!(c.next_value(), Some("hello".as_bytes()));
        assert_eq!(c.next_value(), None);
        assert_eq!(c.next_value(), None);
    }
    
    #[test]
    fn next_val_2() {
        let file = ",\n";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.next_value(), Some("".as_bytes()));
        assert_eq!(c.next_value(), Some("".as_bytes()));
        assert_eq!(c.next_value(), None);
        assert_eq!(c.next_value(), None);
    }
    
    #[test]
    fn next_line() {
        let file = "hi,hello\nlater,bye";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.advance_line(), true);
        assert_eq!(c.advance_line(), false);
        assert_eq!(c.advance_line(), false);
    }
    
    #[test]
    fn next_line_2() {
        let file = "\n\n";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.advance_line(), true);
        assert_eq!(c.advance_line(), true);
        assert_eq!(c.advance_line(), false);
        assert_eq!(c.advance_line(), false);
    }

    #[test]
    fn read_file() {
        let file = "hi,hello\nlater,bye";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.next_value(), Some("hi".as_bytes()));
        assert_eq!(c.next_value(), Some("hello".as_bytes()));
        assert_eq!(c.next_value(), None);
        assert_eq!(c.advance_line(), true);
        assert_eq!(c.next_value(), Some("later".as_bytes()));
        assert_eq!(c.next_value(), Some("bye".as_bytes()));
        assert_eq!(c.next_value(), None);
        assert_eq!(c.advance_line(), false);
    }
    
    #[test]
    fn eof_thrash() {
        let file = ",";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.next_value(), Some("".as_bytes()));
        assert_eq!(c.next_value(), Some("".as_bytes()));
        assert_eq!(c.advance_line(), false);
        assert_eq!(c.advance_line(), false);
        assert_eq!(c.next_value(), None);
        assert_eq!(c.advance_line(), false);
    }
    
    #[test]
    fn empty_file() {
        let file = "";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.next_value(), Some("".as_bytes()));
        assert_eq!(c.advance_line(), false);
    }
    
    #[test]
    fn get_value() {
        let file = "0,1,2,3,4,5";
        let bytes = file.as_bytes();
        let mut  c = CsvSimdCursor::new(bytes);
        assert_eq!(c.get_value(1), Some("1".as_bytes()));
        assert_eq!(c.get_value(3), Some("3".as_bytes()));
        assert_eq!(c.get_value(2), None);
        assert_eq!(c.get_value(4), Some("4".as_bytes()));
        assert_eq!(c.get_value(6), None);
        assert_eq!(c.at_end(), true);
    }
}