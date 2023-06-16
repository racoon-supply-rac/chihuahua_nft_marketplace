#[macro_export]
macro_rules! print_info {
    ($variable:expr) => {
        println!("{:#?}", $variable);
    };
}

#[macro_export]
macro_rules! validate_all_addr {
    ($deps:expr, $vec:expr) => {{
        for addr in $vec.iter() {
            $deps.api.addr_validate(&addr)?;
        }
    }};
}