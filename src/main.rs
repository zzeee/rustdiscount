use serde::{Deserialize, Serialize};
// use reqwest;
mod api;
use api::*;
// use futures::executor::block_on;
use serde_json::json;
use crate::input::Input;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub value: f64,
}

impl Configuration {
    const DEFAULT_VALUE: f64 = 11.00;

    fn from_str(value: &str) -> Self {
        serde_json::from_str(value).expect("Unable to parse configuration value from metafield")
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            value: Self::DEFAULT_VALUE,
        }
    }
}

impl input::Input {
    pub fn configuration(&self) -> Configuration {
        /*match &self.discount_node.metafield {
            Some(input::Metafield { value }) => Configuration::from_str(value),
            None => Configuration::default(),
        }*/
        Configuration::default()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input: input::Input = serde_json::from_reader(std::io::BufReader::new(std::io::stdin()))?;
    let mut out = std::io::stdout();
    let mut serializer = serde_json::Serializer::new(&mut out);
    function(input)?.serialize(&mut serializer)?;
    Ok(())
}

fn function(input: input::Input) -> Result<FunctionResult, Box<dyn std::error::Error>> {
    // let config = input.configuration();
    let converted_value = 23.0; // convert_to_cart_currency(config.value, input.presentment_currency_rate);
    let targets = targets(&input.cart.lines);
    Ok(build_result(converted_value, targets, input))
}

fn convert_to_cart_currency(value: f64, presentment_currency_rate: f64) -> f64 {
    value * presentment_currency_rate
}

fn targets(cart_lines: &[input::CartLine]) -> Vec<Target> {
    cart_lines
        .iter()
        .filter_map(|line| {
            line.merchandise
                .id
                .as_ref()
                .map(|id| Target::ProductVariant {
                    id: id.to_string(),
                    quantity: None,
                })
        })
        .collect()
}

fn build_result(amount: f64, targets: Vec<Target>, input: Input) -> FunctionResult {
    let aresult=json!(input).to_string();
    let discounts = if targets.is_empty() {
        vec![]
    } else {
        vec![Discount {
            message: Some(aresult),
            conditions: None,
            targets,
            value: Value::FixedAmount {
                amount,
                applies_to_each_item: true,
            },
        }]
    };
    FunctionResult {
        discounts,
        discount_application_strategy: DiscountApplicationStrategy::First,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        config: Option<Configuration>,
        presentment_currency_rate: Option<Decimal>,
        cart_lines: Option<Vec<input::CartLine>>,
    ) -> input::Input {
        let lines = cart_lines.unwrap_or_else(||
            vec![
                input::CartLine {
                    id: "gid://shopify/CartLine/0".to_string(),
                    merchandise: input::Merchandise { id: Some("gid://shopify/ProductVariant/0".to_string()) }
                },
                input::CartLine {
                    id: "gid://shopify/CartLine/1".to_string(),
                    merchandise: input::Merchandise { id: Some("gid://shopify/ProductVariant/1".to_string()) }
                },
            ]
        );
        input::Input {
            discount_node: input::DiscountNode {
                metafield: Some(input::Metafield {
                    value: serde_json::to_string(&config.unwrap_or_default()).unwrap()
                }),
            },
            presentment_currency_rate: presentment_currency_rate.unwrap_or(1.00),
            cart: input::Cart { lines },
        }
    }

    #[test]
    fn test_discount_with_no_configuration() {
        let input = input(None, None, None);
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_handle_result = serde_json::json!({
            "discounts": [
                {
                    "targets": [
                        { "productVariant": { "id": "gid://shopify/ProductVariant/0" } },
                        { "productVariant": { "id": "gid://shopify/ProductVariant/1" } },
                    ],
                    "value": { "fixedAmount": { "amount": "50", "appliesToEachItem": false } },
                }
            ],
            "discountApplicationStrategy": "FIRST",
        });
        assert_eq!(handle_result, expected_handle_result);
    }

    #[test]
    fn test_discount_with_value() {
        let input = input(Some(Configuration { value: 12.34 }), None, None);
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_handle_result = serde_json::json!({
            "discounts": [
                {
                    "targets": [
                        { "productVariant": { "id": "gid://shopify/ProductVariant/0" } },
                        { "productVariant": { "id": "gid://shopify/ProductVariant/1" } },
                    ],
                    "value": { "fixedAmount": { "amount": "12.34", "appliesToEachItem": false } },
                }
            ],
            "discountApplicationStrategy": "FIRST",
        });
        assert_eq!(handle_result, expected_handle_result);
    }

    #[test]
    fn test_discount_with_presentment_currency_rate() {
        let input = input(Some(Configuration { value: 10.00 }), Some(2.00), None);
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_handle_result = serde_json::json!({
            "discounts": [
                {
                    "targets": [
                        { "productVariant": { "id": "gid://shopify/ProductVariant/0" } },
                        { "productVariant": { "id": "gid://shopify/ProductVariant/1" } },
                    ],
                    "value": { "fixedAmount": { "amount": "20", "appliesToEachItem": false } },
                }
            ],
            "discountApplicationStrategy": "FIRST",
        });
        assert_eq!(handle_result, expected_handle_result);
    }

    #[test]
    fn test_discount_with_empty_cart_lines() {
        let input = input(None, None, Some(vec![]));
        let handle_result = serde_json::json!(function(input).unwrap());

        let expected_handle_result = serde_json::json!({
            "discounts": [],
            "discountApplicationStrategy": "FIRST",
        });
        assert_eq!(handle_result, expected_handle_result);
    }

    #[test]
    fn test_input_deserialization() {
        let input_json = r#"
        {
            "cart": {
                "lines": [
                    {
                        "id": "gid://shopify/CartLine/0",
                        "merchandise": { "id": "gid://shopify/ProductVariant/0" }
                    }
                ]
            },
            "discountNode": { "metafield": { "value": "{\"value\":10.0}" } },
            "presentmentCurrencyRate": "3.00"
        }
        "#;

        let expected_input = input(
            Some(Configuration { value: 10.00 }),
            Some(3.00),
            Some(vec![
                input::CartLine {
                    id: "gid://shopify/CartLine/0".to_string(),
                    merchandise: input::Merchandise { id: Some("gid://shopify/ProductVariant/0".to_string()) }
                }
            ])
        );
        println!("test_input_deserialization {:?}", expected_input);
        assert_eq!(expected_input, serde_json::from_str::<input::Input>(input_json).unwrap());
    }
}
