// // https://bodil.lol/parser-combinators/

// #![type_length_limit = "16777216"]
// use std::ops::RangeBounds;

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Element {
//     name: String,
//     attributes: Vec<(String, String)>,
//     children: Vec<Element>,
// }

// //Fn(&str) -> Result<(&str, Element), &str>

// // method request-target version
// // GET / HTTP/1.1

// // request_line = zip(method, sp, request_target, sp, http_version, crlf)
// // status_line = zip(version, sp, status_code, sp, status_message, crlf)

// // header = zip(string, colon, string, crlf)
// // body = zero_or_more(byte)

// // case-insensitive field name, ":", zero_or_more(whitespace), field value, zero_or_more(whitespace)

// // request = zip(request_line, zero_or_more(header), crlf, body)
// // response = zip(status_line, zero_or_more(header), crlf, body)

// #[derive(Debug)]
// enum Method {
//     Get,
//     Head,
//     Post,
//     Put,
//     Delete,
//     Connect,
//     Options,
//     Trace,
// }

// fn method(input: &str) -> ParseResult<Method> {
//     loop {
//         match input {

//         }
//     }
// }

// fn zip2<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
// where
//     P1: Parser<'a, R1>,
//     P2: Parser<'a, R2>,
// {
//     move |input| {
//         parser1.parse(input).and_then(|(rest1, result1)| {
//             parser2
//                 .parse(rest1)
//                 .map(|(rest2, result2)| (rest2, (result1, result2)))
//         })
//     }
// }

// fn zip3<'a, P1, P2, P3, R1, R2, R3>(
//     parser1: P1,
//     parser2: P2,
//     parser3: P3,
// ) -> impl Parser<'a, (R1, R2, R3)>
// where
//     P1: Parser<'a, R1>,
//     P2: Parser<'a, R2>,
//     P3: Parser<'a, R3>,
// {
//     move |input| {
//         parser1.parse(input).and_then(|(rest1, result1)| {
//             zip2(&parser2, &parser3)
//                 .parse(rest1)
//                 .map(|(rest2, (result2, result3))| {
//                     (rest2, (result1, result2, result3))
//                 })
//         })
//     }
// }

// fn map<P, F, A, B>(parser: P, function: F) -> impl Parser<B>
// where
//     P: Parser<A>,
//     F: Fn(A) -> B,
// {
//     move |input| {
//         parser
//             .parse(input)
//             .map(|(rest, result)| (rest, function(result)))
//     }
// }

// fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A>
// where
//     P: Parser<'a, A>,
//     F: Fn(&A) -> bool,
// {
//     move |input| {
//         if let Ok((rest, value)) = parser.parse(input) {
//             if predicate(&value) {
//                 return Ok((rest, value));
//             }
//         }
//         Err(input)
//     }
// }

// type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

// trait Parser<'a, Output> {
//     fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

//     fn map<F, NewOutput>(self, function: F) -> BoxedParser<'a, NewOutput>
//     where
//         Self: Sized + 'a,
//         Output: 'a,
//         NewOutput: 'a,
//         F: Fn(Output) -> NewOutput + 'a,
//     {
//         BoxedParser::new(map(self, function))
//     }

//     fn pred<F>(Self, predicate: F) -> BoxedParser<'a, Output>
//     where
//         Self: Sized + 'a,
//         Output: 'a,
//         F: Fn(&Output) -> bool + 'a,
//     {
//         BoxedParser::new(pred(self, predicate))
//     }
// }

// // any closure that takes a &str and returns a ParseResult<Output> is a Parser<Output>
// // and calling some_parser.parse(input) is like doing some_parser()
// impl<'a, F, Output> Parser<'a, Output> for F
// where
//     F: Fn(&'a str) -> ParseResult<Output>,
// {
//     fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
//         self(input)
//     }
// }

// struct BoxedParser<'a, Output> {
//     parser: Box<dyn Parser<'a, Output> + 'a>,
// }

// impl<'a, Output> BoxedParser<'a, Output> {
//     fn new<P>(parser: P) -> self
//     where
//         P: Parser<'a, Output> + 'a,
//     {
//         BoxedParser {
//             parser: Box::new(parser),
//         }
//     }
// }

// impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
//     fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
//         self.parser.parse(input)
//     }
// }

// // parser
// fn the_letter_a(input: &str) -> ParseResult<()> {
//     match input.chars().next() {
//         Some('a') => Ok((&input['a'.len_utf8()..], ())),
//         _ => Err(input),
//     }
// }

// // parser
// fn any_char(input: &str) -> ParseResult<char> {
//     match input.chars().next() {
//         Some(next) => Ok((&input[next.len_utf8()..], next)),
//         _ => Err(input),
//     }
// }

// fn token<'a>() -> impl Parser<'a, char> {
//     zero_or_more(any_char.pred(|c| c.is_alphanumeric || b"!#$%&'*+-.^_`|~".contains(c)))
// }

// fn whitespace_char<'a>() -> impl Parser<'a, char> {
//     pred(any_char, |c| c.is_whitespace())
// }

// fn space1<'a>() -> impl Parser<'a, Vec<char>> {
//     one_or_more(whitespace_char())
// }

// fn space0<'a>() -> impl Parser<'a, Vec<char>> {
//     zero_or_more(whitespace_char())
// }

// fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
//     pair(identifier, right(match_literal("="), quoted_string()))
// }

// fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>> {
//     zero_or_more(right(space1(), attribute_pair()))
// }

// fn quoted_string<'a>() -> impl Parser<'a, String> {
//     right(
//         match_literal("\""),
//         left(
//             zero_or_more(pred(any_char, |c| *c != '"')),
//             match_literal("\""),
//         ),
//     ).map(|chars| chars.into_iter().collect())
// }

// // parser combinator
// fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
//     move |input: &'a str| match input.get(0..expected.len()) {
//         Some(next) if next == expected => Ok((&input[expected.len()..], ())),
//         _ => Err(input),
//     }
// }

// fn identifier(input: &str) -> ParseResult<String> {
//     let mut matched = String::new();
//     let mut chars = input.chars();

//     match chars.next() {
//         Some(next) if next.is_alphabetic() => matched.push(next),
//         _ => return Err(input),
//     }

//     while let Some(next) = chars.next() {
//         if next.is_alphanumeric() || next == '-' {
//             matched.push(next);
//         } else {
//             break;
//         }
//     }

//     let next_index = matched.len();
//     Ok((&input[next_index..], matched))
// }

// fn xx() -> impl Parser<bool> {}

// fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
// where
//     P: Parser<'a, A>,
// {
//     move |mut input| {
//         let mut result = Vec::new();

//         while let Ok((next_input, next_item)) = parser.parse(input) {
//             input = next_input;
//             result.push(next_item);
//         }

//         Ok((input, result))
//     }
// }

// fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
// where
//     P: Parser<'a, A>,
// {
//     let x = move |mut input| {
//         let mut result = Vec::new();

//         if let Ok((next_input, first_item)) = parser.parse(input) {
//             input = next_input;
//             result.push(first_item);
//         } else {
//             return Err(input);
//         }

//         while let Ok((next_input, next_item)) = parser.parse(input) {
//             input = next_input;
//             result.push(next_item);
//         }

//         Ok((input, result))
//     };

//     x
// }

// // fn repeat_n<'a, P, A, R>(parser: P, range: R) -> impl Parser<'a, Vec<A>>
// // where
// //     P: Parser<'a, A>,
// //     R: RangeBounds<u8>,
// // {
// //     move |mut input| {
// //         let mut result = Vec::new();

// //         for n in 0..range.start_bound() {
// //             if let Ok((next_input, next_item)) = parser.parse(input) {
// //                 input = next_input;
// //                 result.push(next_item);
// //             } else {
// //                 return Err(input);
// //             }
// //         }

// //         for n in range {
// //             if let Ok((next_input, next_item)) = parser.parse(input) {
// //                 input = next_input;
// //                 result.push(next_item);
// //             }
// //         }

// //         Ok((input, result))
// //     }
// // }

// fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
// where
//     P1: Parser<'a, R1>,
//     P2: Parser<'a, R2>,
// {
//     zip(parser1, parser2).map(|(left, _)| left)
// }

// fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
// where
//     P1: Parser<'a, R1>,
//     P2: Parser<'a, R2>,
// {
//     zip(parser1, parser2).map(|(_, right)| right)
// }

// #[test]
// fn literal_parser() {
//     let parse_joe = match_literal("Hello Joe!");
//     assert_eq!(Ok(("", ())), parse_joe.parse("Hello Joe!"));
//     assert_eq!(
//         Ok((" Hello Robert!", ())),
//         parse_joe.parse("Hello Joe! Hello Robert!")
//     );
//     assert_eq!(Err("Hello Mike!"), parse_joe.parse("Hello Mike!"));
// }

// #[test]
// fn identifier_parser() {
//     assert_eq!(
//         Ok(("", "i-am-an-identifier".to_string())),
//         identifier("i-am-an-identifier")
//     );
//     assert_eq!(
//         Ok((" entirely an identifier", "not".to_string())),
//         identifier("not entirely an identifier")
//     );
//     assert_eq!(
//         Err("!not at all an identifier"),
//         identifier("!not at all an identifier")
//     );
// }

// #[test]
// fn right_combinator() {
//     let tag_opener = right(match_literal("<"), identifier);
//     assert_eq!(
//         Ok(("/>", "my-first-element".to_string())),
//         tag_opener.parse("<my-first-element/>")
//     );
//     assert_eq!(Err("oops"), tag_opener.parse("oops"));
//     assert_eq!(Err("!oops"), tag_opener.parse("!oops"));
// }

// #[test]
// fn one_or_more_combinator() {
//     let parser = one_or_more(match_literal("ha"));
//     assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
//     assert_eq!(Err("ahah"), parser.parse("ahah"));
//     assert_eq!(Err(""), parser.parse(""));
// }

// #[test]
// fn zero_or_more_combinator() {
//     let parser = zero_or_more(match_literal("ha"));
//     assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
//     assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
//     assert_eq!(Ok(("", vec![])), parser.parse(""));
// }

// // #[test]
// // fn repeat_n_combinator() {
// //     let zero_or_more = repeat_n(match_literal("ha"), (0..));
// //     assert_eq!(Ok(("", vec![(), (), ()])), zero_or_more.parse("hahaha"));
// //     assert_eq!(Ok(("ahah", vec![])), zero_or_more.parse("ahah"));
// //     assert_eq!(Ok(("", vec![])), zero_or_more.parse(""));

// //     let one_or_more = repeat_n(match_literal("ha"), (1..));
// //     assert_eq!(Ok(("", vec![(), (), ()])), one_or_more.parse("hahaha"));
// //     assert_eq!(Err("ahah"), one_or_more.parse("ahah"));
// //     assert_eq!(Err(""), one_or_more.parse(""));

// //     let two_or_three = repeat_n(match_literal("ha"), (2..=3));
// //     assert_eq!(Ok(("", vec![(), ()])), two_or_three.parse("haha"));
// //     assert_eq!(Ok(("", vec![(), (), ()])), two_or_three.parse("hahaha"));
// //     assert_eq!(Ok(("ha", vec![(), (), ()])), two_or_three.parse("hahahaha"));
// //     assert_eq!(Err("ha"), two_or_three.parse("ha"));
// // }

// #[test]
// fn predicate_combinator() {
//     let parser = pred(any_char, |c| *c == 'o');
//     assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
//     assert_eq!(Err("lol"), parser.parse("lol"));
// }

// #[test]
// fn quoted_string_parser() {
//     assert_eq!(
//         Ok(("", "Hello Joe!".to_string())),
//         quoted_string().parse("\"Hello Joe!\"")
//     );
// }

// #[test]
// fn attribute_parser() {
//     assert_eq!(
//         Ok((
//             "",
//             vec![
//                 ("one".to_string(), "1".to_string()),
//                 ("two".to_string(), "2".to_string())
//             ]
//         )),
//         attributes().parse(" one=\"1\" two=\"2\"")
//     );
// }
