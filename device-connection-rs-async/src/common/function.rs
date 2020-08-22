//!
//!
//!


/// Example:
/// ```
/// pub fn main() {
///    (|| {
/// mod module {
///            pub trait Trait {
///                fn function(&self) {
///                    println!("{} (in {} [{}:{}:{}])",
///                             function!(), module_path!(), file!(), line!(), column!()
///                    );
///                }
///            }
///            impl Trait for () {}
///        }
///        module::Trait::function(&());
///    })()
/// }
/// ```

#[macro_export]
macro_rules! functionname {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }}
}
