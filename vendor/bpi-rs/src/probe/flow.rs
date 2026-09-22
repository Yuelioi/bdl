use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::probe::account::RawProbeConfig;
use crate::probe::contract::{ApiContract, ProbeResult};
use crate::probe::run::execute_contract;
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeFlow {
    pub name: String,
    pub steps: Vec<ProbeFlowStep>,
}

impl ProbeFlow {
    pub fn from_slice(bytes: &[u8]) -> BpiResult<Self> {
        let raw: RawProbeFlow = serde_json::from_slice(bytes)?;
        raw.try_into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeFlowStep {
    pub name: String,
    pub contract: serde_json::Value,
    pub extract: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeFlowResult {
    pub flow: String,
    pub steps: Vec<ProbeFlowStepResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeFlowStepResult {
    pub step: String,
    pub result: ProbeResult,
}

#[derive(Debug, Deserialize)]
struct RawProbeFlow {
    name: String,
    steps: Vec<RawProbeFlowStep>,
}

#[derive(Debug, Deserialize)]
struct RawProbeFlowStep {
    name: String,
    contract: serde_json::Value,
    #[serde(default)]
    extract: BTreeMap<String, String>,
}

impl TryFrom<RawProbeFlow> for ProbeFlow {
    type Error = BpiError;

    fn try_from(raw: RawProbeFlow) -> Result<Self, Self::Error> {
        if raw.steps.is_empty() {
            return Err(BpiError::invalid_parameter(
                "steps",
                "flow must contain at least one step",
            ));
        }

        Ok(Self {
            name: raw.name,
            steps: raw
                .steps
                .into_iter()
                .map(|step| ProbeFlowStep {
                    name: step.name,
                    contract: step.contract,
                    extract: step.extract,
                })
                .collect(),
        })
    }
}

pub async fn execute_flow(
    flow: &ProbeFlow,
    accounts: &RawProbeConfig,
) -> BpiResult<ProbeFlowResult> {
    let mut variables = BTreeMap::new();
    let mut steps = Vec::with_capacity(flow.steps.len());

    for step in &flow.steps {
        let contract = contract_for_step(step, &variables)?;
        let result = execute_contract(&contract, accounts).await?;
        extract_variables(step, &result, &mut variables)?;

        steps.push(ProbeFlowStepResult {
            step: step.name.clone(),
            result,
        });
    }

    Ok(ProbeFlowResult {
        flow: flow.name.clone(),
        steps,
    })
}

fn contract_for_step(
    step: &ProbeFlowStep,
    variables: &BTreeMap<String, String>,
) -> BpiResult<ApiContract> {
    let contract = render_value(&step.contract, variables)?;
    ApiContract::from_value(contract)
}

fn extract_variables(
    step: &ProbeFlowStep,
    result: &ProbeResult,
    variables: &mut BTreeMap<String, String>,
) -> BpiResult<()> {
    if step.extract.is_empty() {
        return Ok(());
    }

    let result_value = serde_json::to_value(result)?;
    for (name, pointer) in &step.extract {
        let value = result_value.pointer(pointer).ok_or_else(|| {
            BpiError::unsupported_response(format!(
                "flow step {} missing extraction pointer {}",
                step.name, pointer
            ))
        })?;
        variables.insert(name.clone(), scalar_to_string(name, value)?);
    }

    Ok(())
}

fn scalar_to_string(name: &str, value: &serde_json::Value) -> BpiResult<String> {
    match value {
        serde_json::Value::String(value) => Ok(value.clone()),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        serde_json::Value::Bool(value) => Ok(value.to_string()),
        _ => Err(BpiError::unsupported_response(format!(
            "flow variable {name} must be a string, number, or boolean"
        ))),
    }
}

fn render_value(
    value: &serde_json::Value,
    variables: &BTreeMap<String, String>,
) -> BpiResult<serde_json::Value> {
    match value {
        serde_json::Value::String(value) => {
            Ok(serde_json::Value::String(render_string(value, variables)?))
        }
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| render_value(value, variables))
            .collect::<BpiResult<Vec<_>>>()
            .map(serde_json::Value::Array),
        serde_json::Value::Object(values) => values
            .iter()
            .map(|(key, value)| Ok((key.clone(), render_value(value, variables)?)))
            .collect::<BpiResult<serde_json::Map<_, _>>>()
            .map(serde_json::Value::Object),
        value => Ok(value.clone()),
    }
}

fn render_string(input: &str, variables: &BTreeMap<String, String>) -> BpiResult<String> {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("${") {
        output.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        let end = after_open.find('}').ok_or_else(|| {
            BpiError::invalid_parameter("template", "missing closing brace in flow variable")
        })?;
        let name = &after_open[..end];
        let value = variables.get(name).ok_or_else(|| {
            BpiError::invalid_parameter("template", "flow variable is not defined")
        })?;
        output.push_str(value);
        rest = &after_open[end + 1..];
    }

    output.push_str(rest);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::{CapturedRequest, HttpMethod, ProbeResponse};

    #[test]
    fn flow_rejects_empty_steps() {
        let err = ProbeFlow::from_slice(br#"{ "name": "empty", "steps": [] }"#).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "steps", .. }
        ));
    }

    #[test]
    fn flow_contract_renders_variables_in_nested_strings() -> Result<(), BpiError> {
        let step = ProbeFlowStep {
            name: "poll".to_string(),
            contract: serde_json::json!({
                "name": "login.qr_poll.anonymous",
                "request": {
                    "method": "GET",
                    "url": "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
                    "query": {
                        "qrcode_key": "${qrcode_key}",
                        "source": "web-${source}"
                    },
                    "body": {
                        "nested": ["${qrcode_key}"]
                    }
                },
                "expect": {
                    "api_code": 0
                }
            }),
            extract: BTreeMap::new(),
        };
        let variables = BTreeMap::from([
            ("qrcode_key".to_string(), "key-1".to_string()),
            ("source".to_string(), "main".to_string()),
        ]);

        let contract = contract_for_step(&step, &variables)?;

        assert_eq!(contract.request.query["qrcode_key"], "key-1");
        assert_eq!(contract.request.query["source"], "web-main");
        assert_eq!(contract.request.body.unwrap()["nested"][0], "key-1");
        Ok(())
    }

    #[test]
    fn flow_contract_rejects_missing_variable() {
        let step = ProbeFlowStep {
            name: "poll".to_string(),
            contract: serde_json::json!({
                "name": "login.qr_poll.anonymous",
                "request": {
                    "method": "GET",
                    "url": "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
                    "query": {
                        "qrcode_key": "${qrcode_key}"
                    }
                }
            }),
            extract: BTreeMap::new(),
        };

        let err = contract_for_step(&step, &BTreeMap::new()).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "template",
                ..
            }
        ));
    }

    #[test]
    fn flow_extracts_variables_from_probe_result() -> Result<(), BpiError> {
        let step = ProbeFlowStep {
            name: "generate".to_string(),
            contract: serde_json::json!({}),
            extract: BTreeMap::from([(
                "qrcode_key".to_string(),
                "/response/body/data/qrcode_key".to_string(),
            )]),
        };
        let result = ProbeResult {
            contract: "login.qr_generate.anonymous".to_string(),
            request: CapturedRequest {
                method: HttpMethod::Get,
                url: "https://passport.bilibili.com/x/passport-login/web/qrcode/generate"
                    .to_string(),
                headers: BTreeMap::new(),
                query: BTreeMap::new(),
                body: None,
            },
            response: ProbeResponse {
                status: 200,
                headers: BTreeMap::new(),
                body: serde_json::json!({
                    "code": 0,
                    "data": {
                        "qrcode_key": "key-1"
                    }
                }),
            },
        };
        let mut variables = BTreeMap::new();

        extract_variables(&step, &result, &mut variables)?;

        assert_eq!(variables["qrcode_key"], "key-1");
        Ok(())
    }
}
