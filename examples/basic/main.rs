use enum_variant_kind_derive::EnumVariantKind;

#[derive(EnumVariantKind, Debug, Clone, Copy, PartialEq, Eq)]
enum GrammarElement {
    Expr,
    Factor,
    Term,
}

fn main() -> Result<(), String> {
    let variants = [
        GrammarElement::Expr,
        GrammarElement::Factor,
        GrammarElement::Term,
    ];

    let variant_kinds = [
        GrammarElementKind::Expr,
        GrammarElementKind::Factor,
        GrammarElementKind::Term,
    ];

    assert_eq!(
        variant_kinds.to_vec(),
        variants
            .into_iter()
            .map(|v| v.as_variant_kind())
            .collect::<Vec<_>>()
    );

    Ok(())
}
