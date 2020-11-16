
#[derive(Debug)]
pub struct PSym {
    pub start : usize,
    pub end : usize,
    pub value : String,
}

#[derive(Debug)]
pub enum ParseError {
    EndOfFile(String),
    ErrorAt(usize, String),
}


pub struct Input<'a> {
    data : &'a [(usize, char)] 
}

#[derive(Clone, Copy)]
pub struct RestorePoint<'a> {
    data : &'a [(usize, char)] 
}

impl<'a> Input<'a> {

    pub fn new(input : &'a [(usize, char)] ) -> Input<'a> { 
        Input { data: input }
    }

    pub fn expect_end(&mut self) -> Result<(), ParseError> {
        self.clear()?;

        match self.data {
            [] => Ok(()),
            [(i,c), ..] => Err(ParseError::ErrorAt(*i, format!("Expected end of input, but found {:?}", *c))),
        }
    }

    pub fn clear(&mut self) -> Result<(), ParseError> { 
        let mut d = self.data;
        let mut comment = 0;
        loop {
            match d {
                [] if comment > 0 => return Err(ParseError::EndOfFile("Expected end of comment but found end of file".to_string())),
                [] => break,
                [(_, '/'), (_, '*'), rest @ ..] => {
                    comment += 1;
                    d = rest; 
                },
                [(_, '*'), (_, '/'), rest @ ..] if comment > 0 => {
                    comment -= 1;
                    d = rest; 
                }, 
                [_, rest @ ..] if comment > 0 => d = rest,
                [(_, x), rest @ ..] if x.is_whitespace() => d = rest,
                _ => break,
            }
        }
        self.data = d;
        Ok(())
    }

    pub fn create_restore(&self) -> RestorePoint<'a> {
        RestorePoint{ data: self.data }
    }

    pub fn restore(&mut self, restore_point : RestorePoint<'a>) {
        self.data = restore_point.data 
    }

    pub fn expect(&mut self,  s : &str) -> Result<(), ParseError>  {
        self.clear()?;

        let mut d = self.data;
        for c in s.chars() {
            match d {
                [] => return Err(ParseError::EndOfFile(format!("Expected {} in {}", c, s))),
                [(_, x), rest @ ..] if *x == c => d = rest,
                [(i, x), ..] => return Err(ParseError::ErrorAt(*i, format!("Expected {} in {} but found {}", c, s, x))),
            }
        }
        self.data = d;
        Ok(())
    }

    pub fn parse_symbol(&mut self) -> Result<PSym, ParseError> {
        self.clear()?;

        let mut d = self.data;
        let mut cs = vec![];
        let start : usize;
        let mut end;

        match d {
            [] => return Err(ParseError::EndOfFile("parse_symbol".to_string())),
            [(i, x), rest @ ..] if x.is_alphabetic() || *x == '_' => {
                d = rest;
                cs.push(x);
                start = *i;
                end = start;
            },
            [(i, x), ..] => return Err(ParseError::ErrorAt(*i, format!("Encountered {} in parse_symbol", x))),
        }

        loop {
            match d {
                [] => break,
                [(i, x), rest @ ..] if x.is_alphanumeric() || *x == '_' => {
                    d = rest;
                    cs.push(x);
                    end = *i;
                },
                [_, ..] => break,
            }
        }

        self.data = d;

        Ok( PSym { start, end, value: cs.into_iter().collect::<String>() } )
    }

    

    pub fn parse_number(&mut self) -> Result<PSym, ParseError> { 
        self.clear()?;
        
        let mut d = self.data;
        let mut cs = vec![];
        let start : usize;
        let mut end;

        match d {
            [] => return Err(ParseError::EndOfFile("parse_number".to_string())),
            [(i, x), rest @ ..] if x.is_numeric() 
                                || *x == '-' => {
                d = rest;
                cs.push(x);
                start = *i;
                end = start;
            },
            [(i, x), ..] => return Err(ParseError::ErrorAt(*i, format!("Encountered {} in parse_number", x))),
        }

        loop {
            match d {
                [] => break, 
                [(i, x), rest @ ..] if x.is_numeric() 
                                    || *x == '.' 
                                    || *x == '-' 
                                    || *x == 'E'
                                    || *x == 'e' => {
                    d = rest;
                    cs.push(x);
                    end = *i;
                },
                [_, ..] => break, 
            }
        }

        if !cs.last().unwrap().is_numeric() {
           Err( ParseError::ErrorAt(end, "parse_number requires last character to be a numeric".to_string()) ) 
        }
        else {
            self.data = d;

            Ok( PSym { start, end, value: cs.into_iter().collect::<String>() } )
        }
    }

    pub fn parse_string(&mut self) -> Result<PSym, ParseError> {
        self.clear()?;

        let mut d = self.data;
        let mut cs = vec![];
        let start : usize;
        let end : usize;

        match d {
            [] => return Err(ParseError::EndOfFile("parse_string".to_string())),
            [(i, '"'), rest @ ..] => {
                d = rest;
                start = *i;
            },
            [(i, x), ..] => return Err(ParseError::ErrorAt(*i, format!("Encountered {} at the beginning of parse_string", x))),
        }

        let mut escape = false;
        loop {
            match d {
                [] => return Err(ParseError::EndOfFile("parse_string".to_string())),
                [(_, '\\'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('\\');
                },
                [(_, 'n'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('\n');
                },
                [(_, 'r'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('\r');
                },
                [(_, '0'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('\0');
                },
                [(_, 't'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('\t');
                },
                [(_, '"'), rest @ ..] if escape => {
                    escape = false;
                    d = rest;
                    cs.push('"');
                },
                [(i, x), ..] if escape => return Err(ParseError::ErrorAt(*i, format!("Encountered unknown escape character {}", x))),
                [(_, '\\'), rest @ ..] => {
                    escape = true;
                    d = rest;
                },
                [(_, '"'), ..] => break,
                [(_, x), rest @ ..] => {
                    d = rest;
                    cs.push(*x);
                },
            }
        }

        match d {
            [] => return Err(ParseError::EndOfFile("parse_string".to_string())),
            [(i, '"'), rest @ ..] => {
                d = rest;
                end = *i;
            },
            [(i, x), ..] => return Err(ParseError::ErrorAt(*i, format!("Encountered {} at the ending of parse_string", x))),
        }

        self.data = d;

        Ok( PSym { start, end, value: cs.into_iter().collect::<String>() } )
    }

    pub fn maybe<T>(&mut self, parse : fn(&mut Input) -> Result<T, ParseError>) -> Option<T> {
        let rp = self.create_restore(); 
        match parse(self) {
            Ok(v) => Some(v),
            Err(_) => { 
                self.restore(rp);
                None 
            },
        }
    }

    pub fn zero_or_more<T>(&mut self, parse : fn(&mut Input) -> Result<T, ParseError>) -> Result<Vec<T>, ParseError> {
        let mut items = vec![];

        loop {
            let rp = self.create_restore(); 
            match parse(self) {
                Ok(v) => items.push(v),
                Err(_) => {
                    self.restore(rp);
                    break
                },
            }
        }

        Ok(items)
    }

    pub fn one_or_more<T>(&mut self, parse : fn(&mut Input) -> Result<T, ParseError>) -> Result<Vec<T>, ParseError> {
        let mut items = vec![];

        items.push( parse(self)? );

        loop {
            let rp = self.create_restore(); 
            match parse(self) {
                Ok(v) => items.push(v),
                Err(_) => {
                    self.restore(rp); 
                    break
                },
            }
        }

        Ok(items)
    }

    pub fn list<T>(&mut self, parse : fn(&mut Input) -> Result<T, ParseError>) -> Result<Vec<T>, ParseError> {
        let mut items = vec![];

        // check to see if this is an empty list
        let rp = self.create_restore();
        match parse(self) {
            Ok(item) => items.push(item),
            Err(_) => {
                self.restore(rp); 
                return Ok(vec![]);
            },
        }

        loop {
            match self.expect(",") {
                Ok(_) => (),
                Err(_) => break,
            }
            items.push(parse(self)?);
        }

        Ok(items)
    }

    pub fn choice<T>(&mut self, parsers : &[fn(&mut Input) -> Result<T, ParseError>]) -> Result<T, ParseError> {

        assert!( parsers.len() > 0, "choice must have at least one parser" );

        let mut e = None;
        for parse in parsers.iter() {
            let rp = self.create_restore();
            match parse(self) {
                Ok(item) => return Ok(item),
                Err(err) => {
                    e = Some(err);
                    self.restore(rp);
                },
            }
        }

        Err(e.expect("Encountered choice with zero successes and zero failures"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_single_character_symbol_second() -> Result<(), ParseError> {
        let mut input = Input { data: &"::<>:: b d".char_indices().collect::<Vec<(usize, char)>>() };
        input.expect("::<>::")?;
        let sym = input.parse_symbol()?;
        assert_eq!( sym.value, "b" );
        assert_eq!( sym.start, 7 );
        assert_eq!( sym.end, 7 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " d".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_double_character_symbol_second() -> Result<(), ParseError> {
        let mut input = Input { data: &"::<>:: bb d".char_indices().collect::<Vec<(usize, char)>>() };
        input.expect("::<>::")?;
        let sym = input.parse_symbol()?;
        assert_eq!( sym.value, "bb" );
        assert_eq!( sym.start, 7 );
        assert_eq!( sym.end, 8 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " d".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_single_digit_number_second() -> Result<(), ParseError> {
        let mut input = Input { data: &"::<>:: 1 d".char_indices().collect::<Vec<(usize, char)>>() };
        input.expect("::<>::")?;
        let sym = input.parse_number()?;
        assert_eq!( sym.value, "1" );
        assert_eq!( sym.start, 7 );
        assert_eq!( sym.end, 7 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " d".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_double_digit_number_second() -> Result<(), ParseError> {
        let mut input = Input { data: &"::<>:: 11 d".char_indices().collect::<Vec<(usize, char)>>() };
        input.expect("::<>::")?;
        let sym = input.parse_number()?;
        assert_eq!( sym.value, "11" );
        assert_eq!( sym.start, 7 );
        assert_eq!( sym.end, 8 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " d".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_expect_string() -> Result<(), ParseError> {
        let mut input = Input { data: &"::<>::".char_indices().collect::<Vec<(usize, char)>>() };
        input.expect("::<>::")?;
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_symbol() -> Result<(), ParseError> {
        let mut input = Input { data: &"_Symbol_123".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: symbol } = input.parse_symbol()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 10 );
        assert_eq!( symbol, "_Symbol_123" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_clear_whitespace() -> Result<(), ParseError> {
        let mut input = Input { data: &"   x".char_indices().collect::<Vec<(usize, char)>>() };
        input.clear()?;
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_clear_block_comment() -> Result<(), ParseError> {
        let mut input = Input { data: &r#"  
        
        /* comments %^& 124

        */
        
        x"#.char_indices().collect::<Vec<(usize, char)>>() };
        input.clear()?;
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_clear_nested_block_comment() -> Result<(), ParseError> {
        let mut input = Input { data: &r#"  
        
        /* comments %^& 124

            /* nested */
            /* other nest */
            /* /* nest nest */ */

        */
        
        x"#.char_indices().collect::<Vec<(usize, char)>>() };
        input.clear()?;
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_int() -> Result<(), ParseError> {
        let mut input = Input { data: &"1234".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 3 );
        assert_eq!( number, "1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_float() -> Result<(), ParseError> {
        let mut input = Input { data: &"12.34".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "12.34" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_scientific_notation() -> Result<(), ParseError> {
        let mut input = Input { data: &"1234e42.0".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 8 );
        assert_eq!( number, "1234e42.0" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_negative_scientific_notation() -> Result<(), ParseError> {
        let mut input = Input { data: &"1234E-42".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 7 );
        assert_eq!( number, "1234E-42" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_negative_int() -> Result<(), ParseError> {
        let mut input = Input { data: &"-1234".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_string_with_whitespace() -> Result<(), ParseError> {
        let mut input = Input { data: &r#" /* */ " string with 123
whitespace ""#.char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_string()?;
        assert_eq!( start, 7 );
        assert_eq!( end, 36 );
        assert_eq!( number, " string with 123\nwhitespace " );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_string_with_escapes() -> Result<(), ParseError> {
        let mut input = Input { data: &r#" /* */ "\\ \0 \n \r \t \"""#.char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.parse_string()?;
        assert_eq!( start, 7 );
        assert_eq!( end, 25 );
        assert_eq!( number, "\\ \0 \n \r \t \"" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_restore() -> Result<(), ParseError> {
        let mut input = Input { data: &"-1234".char_indices().collect::<Vec<(usize, char)>>() };
        let r = input.create_restore();
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 

        let number = input.parse_number();
        assert_eq!( matches!(number, Err(_)), true );

        input.restore(r);
        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(()) 
    }

    #[test]
    fn should_handle_multiple_restores() -> Result<(), ParseError> {
        let mut input = Input { data: &"-1234 789".char_indices().collect::<Vec<(usize, char)>>() };
        let r1 = input.create_restore();

        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " 789".to_string() ); 

        let r2 = input.create_restore();

        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 6 );
        assert_eq!( end, 8 );
        assert_eq!( number, "789" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 

        input.restore(r2);

        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 6 );
        assert_eq!( end, 8 );
        assert_eq!( number, "789" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 

        input.restore(r1);

        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), " 789".to_string() ); 

        let PSym { start, end, value: number } = input.parse_number()?;
        assert_eq!( start, 6 );
        assert_eq!( end, 8 );
        assert_eq!( number, "789" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 

        Ok(()) 
    }

    #[test]
    fn should_parse_maybe_parser() -> Result<(), ParseError> {
        let mut input = Input { data: &"-1234".char_indices().collect::<Vec<(usize, char)>>() };
        let PSym { start, end, value: number } = input.maybe(|i| i.parse_number()).unwrap();
        assert_eq!( start, 0 );
        assert_eq!( end, 4 );
        assert_eq!( number, "-1234" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_maybe_parser_with_nothing() -> Result<(), ParseError> {
        let mut input = Input { data: &"x".char_indices().collect::<Vec<(usize, char)>>() };
        let number = input.maybe(|i| i.parse_number());
        match number {
            None => (),
            _ => panic!( "nothing should be parsed" ), 
        }
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_zero_or_more_with_some_items() -> Result<(), ParseError> {
        let mut input = Input { data: &"123 456".char_indices().collect::<Vec<(usize, char)>>() };
        let numbers = input.zero_or_more(|i| i.parse_number())?;
        assert_eq!( numbers.len(), 2 );
        assert_eq!( numbers[0].value, "123" );
        assert_eq!( numbers[1].value, "456" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_zero_or_more_with_no_items() -> Result<(), ParseError> {
        let mut input = Input { data: &"x".char_indices().collect::<Vec<(usize, char)>>() };
        let numbers = input.zero_or_more(|i| i.parse_number())?;
        assert_eq!( numbers.len(), 0 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_one_or_more_with_some_items() -> Result<(), ParseError> {
        let mut input = Input { data: &"123 456".char_indices().collect::<Vec<(usize, char)>>() };
        let numbers = input.one_or_more(|i| i.parse_number())?;
        assert_eq!( numbers.len(), 2 );
        assert_eq!( numbers[0].value, "123" );
        assert_eq!( numbers[1].value, "456" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_one_or_more_with_one_item() -> Result<(), ParseError> {
        let mut input = Input { data: &"123".char_indices().collect::<Vec<(usize, char)>>() };
        let numbers = input.one_or_more(|i| i.parse_number())?;
        assert_eq!( numbers.len(), 1 );
        assert_eq!( numbers[0].value, "123" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_fail_one_or_more_with_no_item() -> Result<(), ParseError> {
        let mut input = Input { data: &"x".char_indices().collect::<Vec<(usize, char)>>() };
        let numbers = input.one_or_more(|i| i.parse_number());
        match numbers {
            Ok(_) => panic!( "one or more should fail on no items" ),
            Err(_) => (),
        }
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_empty_list() -> Result<(), ParseError> {
        let mut input = Input { data: &"x".char_indices().collect::<Vec<(usize, char)>>() };
        let items = input.list(|i| i.parse_number())?;
        assert_eq!( items.len(), 0 );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "x".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_single_item_list() -> Result<(), ParseError> {
        let mut input = Input { data: &"123".char_indices().collect::<Vec<(usize, char)>>() };
        let items = input.list(|i| i.parse_number())?;
        assert_eq!( items.len(), 1 );
        assert_eq!( items[0].value, "123" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_list() -> Result<(), ParseError> {
        let mut input = Input { data: &"123, 456, 789".char_indices().collect::<Vec<(usize, char)>>() };
        let items = input.list(|i| i.parse_number())?;
        assert_eq!( items.len(), 3 );
        assert_eq!( items[0].value, "123" );
        assert_eq!( items[1].value, "456" );
        assert_eq!( items[2].value, "789" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_first_choice() -> Result<(), ParseError> {
        let mut input = Input { data: &"123".char_indices().collect::<Vec<(usize, char)>>() };
        let item = input.choice(&[ |i : &mut Input| -> Result<PSym, ParseError> { i.parse_number() }
                                , |i : &mut Input| -> Result<PSym, ParseError> { i.parse_symbol() }
                                ])?;
        assert_eq!( item.value, "123" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }

    #[test]
    fn should_parse_second_choice() -> Result<(), ParseError> {
        let mut input = Input { data: &"blah".char_indices().collect::<Vec<(usize, char)>>() };
        let item = input.choice(&[ |i : &mut Input| -> Result<PSym, ParseError> { i.parse_number() }
                                , |i : &mut Input| -> Result<PSym, ParseError> { i.parse_symbol() }
                                ])?;
        assert_eq!( item.value, "blah" );
        assert_eq!( input.data.into_iter().map(|(_,x)| x).collect::<String>(), "".to_string() ); 
        Ok(())
    }
}
