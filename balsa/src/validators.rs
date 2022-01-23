use regex::Regex;

// TODO: doesn't reject values like `rgb(355, 255, 255)`
/// RegEx for matching CSS color types (hex, rgb, hsl, color names).
///
/// Thanks to Olmo Kramer and Anton Frattaroli!
/// Slightly modified from: https://gist.github.com/olmokramer/82ccce673f86db7cda5e#gistcomment-3227016
const CSS_COLOR_REGEX: &str = r"^(#(?:[0-9a-f]{2}){2,4}$|(#[0-9a-f]{3}$)|(rgb|hsl)a?\((-?\d+%?[,\s]+){2,3}\s*[\d\.]+%?\)$|black$|silver$|gray$|whitesmoke$|maroon$|red$|purple$|fuchsia$|green$|lime$|olivedrab$|yellow$|navy$|blue$|teal$|aquamarine$|orange$|aliceblue$|antiquewhite$|aqua$|azure$|beige$|bisque$|blanchedalmond$|blueviolet$|brown$|burlywood$|cadetblue$|chartreuse$|chocolate$|coral$|cornflowerblue$|cornsilk$|crimson$|currentcolor$|darkblue$|darkcyan$|darkgoldenrod$|darkgray$|darkgreen$|darkgrey$|darkkhaki$|darkmagenta$|darkolivegreen$|darkorange$|darkorchid$|darkred$|darksalmon$|darkseagreen$|darkslateblue$|darkslategray$|darkslategrey$|darkturquoise$|darkviolet$|deeppink$|deepskyblue$|dimgray$|dimgrey$|dodgerblue$|firebrick$|floralwhite$|forestgreen$|gainsboro$|ghostwhite$|goldenrod$|gold$|greenyellow$|grey$|honeydew$|hotpink$|indianred$|indigo$|ivory$|khaki$|lavenderblush$|lavender$|lawngreen$|lemonchiffon$|lightblue$|lightcoral$|lightcyan$|lightgoldenrodyellow$|lightgray$|lightgreen$|lightgrey$|lightpink$|lightsalmon$|lightseagreen$|lightskyblue$|lightslategray$|lightslategrey$|lightsteelblue$|lightyellow$|limegreen$|linen$|mediumaquamarine$|mediumblue$|mediumorchid$|mediumpurple$|mediumseagreen$|mediumslateblue$|mediumspringgreen$|mediumturquoise$|mediumvioletred$|midnightblue$|mintcream$|mistyrose$|moccasin$|navajowhite$|oldlace$|olive$|orangered$|orchid$|palegoldenrod$|palegreen$|paleturquoise$|palevioletred$|papayawhip$|peachpuff$|peru$|pink$|plum$|powderblue$|rosybrown$|royalblue$|saddlebrown$|salmon$|sandybrown$|seagreen$|seashell$|sienna$|skyblue$|slateblue$|slategray$|slategrey$|snow$|springgreen$|steelblue$|tan$|thistle$|tomato$|transparent$|turquoise$|violet$|wheat$|white$|yellowgreen$|rebeccapurple$)";

/// Validates that a color matches a CSS-accepted color standard.
pub(crate) fn is_valid_color(color: &str) -> bool {
    let regex =
        Regex::new(CSS_COLOR_REGEX).expect("error parsing CSS color regex for `validate_color`");

    regex.is_match(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_color() {
        let valid_colors = vec![
            "#ffffff",
            "orange",
            "rgb(0 , 0,2)",
            "rgba(12, 255, 183 , 0.8)",
            "#ffffffff",
            "hsl(0,0,0)",
            "hsla(0,123,244,0.2)",
            "purple",
        ];
        let invalid_colors = vec![
            "#lololl",
            "#fffffffff",
            "a  rgb(255,255,255)",
            "rustcolor",
            "rbg(0,0,0)",
        ];

        for color in valid_colors {
            assert!(
                is_valid_color(color),
                "`is_valid_color` incorrectly returned `false` for valid color `{}`",
                color
            );
        }

        for color in invalid_colors {
            assert!(
                !is_valid_color(color),
                "`is_valid_color` incorrectly returned `true` for invalid color `{}`",
                color
            );
        }
    }
}
