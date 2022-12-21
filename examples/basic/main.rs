use enum_variant_kind_derive::EnumVariantKind;

#[derive(EnumVariantKind, Debug, Clone, Copy, PartialEq, Eq)]
enum GrammarElements {
    Expr,
    Factor,
    Term,
}

fn main() -> Result<(), String> {
    println!("{:?}", GrammarElementsKind::Expr);

    Ok(())
}
