pub fn keyword_enum_values(name: &str) -> Option<&'static[&'static str]>{
    Some(match name {
        "align" => &["start", "center", "end", "stretch"],
        "justify" => &[ "start", "center" , "end", "spaceBetween", "spaceAround"],
        "fontWeight" => &[ "normal", "bold", "light"],
        "borderStyle" => &[ "solid", "dashed", "dotted", "none"],
        "heading" => &["h1", "h2", "h3", "h4", "h5", "h6"],
        "semantic" => &["header", "footer", "main", "section", "article", "aside", "nav"],
        _ => return None,
    })
}

pub fn is_key_word_enum_property(name: &str) -> bool {
    keyword_enum_values(name).is_some()
}

pub fn css_prop_for(name: &str) -> Option<& 'static str> {
    Some(match name {
        "padding" => "padding","margin" => "margin","spacing" => "gap",
        "color" => "color", "background" => "background-color","fontSize" => "font-size",
        "radius" => "border-radius", "width" => "width", "height" => "height",
        "columns" => "grid-template-columns", "rows" => "grid-template-rows",
        "borderWidth" => "border-width", "borderColor" => "border-color",
        _ =>  return None,
    })
}

pub fn is_animatable_property(name: &str) -> bool {
    matches!(name,
             "padding" | "margin" | "spacing" | "color" | "background" | "fontSize"
             | "radius" | "width" | "height" | "borderWidth" | "borderColor"
                
                
                )
}
//Notes: Call-site updates(each a deletion + one import)
// In p-typeck/src/check.rs-------- delete the local keyword_enum_values function;
// check_single_property calls p_props::keyword_enum_values instead
// Also p-sema/src/sema.rs - delete both and replace them 
// also in p-ir/src/lower.rs 
//Also in p-codegen-css/src/gen.rs

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyword_and_bool_check_stay_consistent_by_construction(){
        assert!(is_key_word_enum_property("heading"));
        assert!(!is_key_word_enum_property("padding"));
    }
}