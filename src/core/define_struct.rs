/// Define a new **struct** with provided field.
///
/// Struct comes with **factory methods** and **traits for shortcut field access**.
/// Shortcut traits should only be implemented for *unambiguous* use.
///
/// # Syntax
///
/// ```text
/// define_struct!(
///     StructName
///     [has { shortcut_trait_path | shortcut_field : Type, ... }]
///     [with { field : Type, ... }]
///     [check { block }]
/// );
/// ```
///
/// * 'has' *(Optional)* - Struct defined using current macro that this struct implements shortcut traits for.
/// * 'with' *(Optional)* - Remaining field including std type and struct defined using current macro *(doesn't implements shortcut traits for them)*.
/// * 'check' *(Optional)* - Field check for factory methods.
///
/// # Examples
/// ```
/// define_struct!(
/// Transaction has{
///     crate::core | label: Label,
///     crate::core | amount: Amount,
///     crate::core | date: Date,
/// } with{
///     group: Group,
///     category: Category,
///     fund: Fund,
///     flow: Flow,
///     link: Option<Label>,
/// });
/// ```
///
#[macro_export]
macro_rules! define_struct {
    (
        $struct_name:ident
        $(has {
            $($field_trait_path:path | $field_trait_name:ident : $field_trait_type:ty),+ $(,)?
        })?
        $(with {
            $($field_name:ident : $field_type:ty),+ $(,)?
        })?
        $(check $new_check:block)?
    ) => {
        // Define new struct
        #[derive(Clone, Debug)]
        pub struct $struct_name {
            $($(pub $field_trait_name : $field_trait_type,)+)?
            $($(pub $field_name : $field_type,)+)?
        }

        // Define shortcut trait
        paste::paste!{
            pub trait [<Has $struct_name>] {
                fn [<$struct_name:lower>](&self) -> &$struct_name;

                $($(
                    fn $field_name(&self) -> &$field_type {
                        &self.[<$struct_name:lower>]().$field_name
                    }
                )+)?
            }
        }

        // Implement factory method
        impl $struct_name {
            pub fn new(
                $($(    $field_trait_name  : $field_trait_type    ,)+)?
                $($(    $field_name : $field_type,    )+)?
            ) -> std::result::Result<$struct_name, String> {
                $($new_check)?

                Ok($struct_name {
                    $($(    $field_trait_name,    )+)?
                    $($(    $field_name,    )+)?
                })
            }
        }

        // Implement shortcut traits
        paste::paste!{
            $($(
                impl $field_trait_path::[<Has $field_trait_type>] for $struct_name {
                    fn [<$field_trait_name:lower>](&self) -> &$field_trait_type {
                        &self.$field_trait_name
                    }
                }
            )+)?
        }
    };
}
