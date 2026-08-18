use p_ast::ElementKind;

pub fn html_tag(kind: ElementKind)-> &'static str{
    use ElementKind::*;
    match kind {
        Row|Column|Stack|Container|Card|Grid => "div",
        List => "ul",
        Table => "table",
        Text => "span",
        Image => "img",
        Icon => "span",
        Input => "input",
        Textarea => "textarea",
        Button => "button",
        Checkbox|Switch|Radio => "input",
        Dropdown => "select",
        Navigation => "nav",
        Tabs => "div",
        Dialog => "dialog",
        Modal => " div",
        Menu => "div",
        Slot => "div",
    }
}

pub fn attr_name(kind:ElementKind) -> &'static str {
    use ElementKind::*;
    match kind {
        Row => "row",
        Column => "column",
        Stack => "stack",
        Container => "container",
        Card => "card",
        Grid => "grid",
        List => "list",
        Text => "text",
        Image => "image",
        Icon => "icon",
        Input => "input",
        Textarea => "textarea",
        Button => "button",
        Checkbox => "checkbox",
        Switch => "switch",
        Radio => "radio",
        Dropdown => "dropdown",
        Table => "table",
        Dialog => "dialog",
        Modal => "modal",
        Menu => "menu",
        Slot => "Slot",
        Navigation => "navigation",
        Tabs => "tabs"

    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn every_kind_maps_both_ways(){
        use ElementKind::*;
        for k in [Row, Column,Stack,Container,Card,Grid,
                               List,Table,Text,Image,Icon,Input,Textarea,Button,
                               Checkbox, Switch, Radio, Dropdown, Navigation,Tabs,Dialog,
                               Modal, Menu,Slot]{
                                let _ = html_tag(k);
                                let _ = attr_name(k);
                               }
                               

                               }
    }
