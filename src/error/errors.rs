use super::*;

//used to generate all the error structs
macro_rules! create_errors
{
    ($($ecode:ident: $message:expr,)*) => (
        pub static DIAGNOSTICS: &[(&str, &str)] = &[
            $( (stringify!($ecode), $message), )*
        ];
    )

}
