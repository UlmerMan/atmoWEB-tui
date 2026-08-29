use std::error::Error;

use reqwest::Client;
use serde_json::Value;

#[derive(Debug)]
pub struct AtmoWeb {
    ip_address: String,
    client: Client,
}

impl AtmoWeb {
    pub fn new(ip_address: impl Into<String>) -> Self {
        AtmoWeb {
            ip_address: ip_address.into(),
            client: Client::new(),
        }
    }

    pub async fn is_online(&self) -> bool {
        let url = format!("http://{}/atmoweb?SN=", self.ip_address);

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(text) => !text.trim().is_empty(),
                Err(_) => false,
            },
            _ => false,
        }
    }

    pub async fn get_ip_address(&self) -> String {
        self.ip_address.clone()
    }

    pub async fn query(&self, params: &[(&str, Option<&str>)]) -> Result<Value, Box<dyn Error>> {
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v.unwrap_or("")))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("http://{}/atmoweb?{}", self.ip_address, query_string);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let json = format!("{{{}}}", text.trim().trim_end_matches(','));

        match serde_json::from_str(&json) {
            Ok(value) => Ok(value),
            Err(_) => Ok(Value::from(json)),
        }
    }

    pub async fn set_temp(&self, celsius: f32) -> Result<f32, Box<dyn Error>> {
        let val = celsius.to_string();
        let resp = self.query(&[("TempSet", Some(val.as_str()))]).await?;
        Ok(resp["TempSet"].as_f64().unwrap() as f32)
    }

    pub async fn read_temp1(&self) -> Result<f64, Box<dyn Error>> {
        let resp = self.query(&[("Temp1Read", None)]).await?;
        Ok(resp["Temp1Read"].as_f64().unwrap())
    }

    pub async fn read_flap(&self) -> Result<f64, Box<dyn Error>> {
        let resp = self.query(&[("FlapSet", None)]).await?;
        Ok(resp["FlapSet"].as_f64().unwrap())
    }

    pub async fn set_flap(&self, value: f64) -> Result<f64, Box<dyn Error>> {
        let val = value.to_string();
        let resp = self.query(&[("FlapSet", Some(val.as_str()))]).await?;
        Ok(resp["FlapSet"].as_f64().unwrap())
    }

    pub async fn read_fan(&self) -> Result<f64, Box<dyn Error>> {
        let resp = self.query(&[("FanRead", None)]).await?;
        Ok(resp["FanRead"].as_f64().unwrap())
    }

    pub async fn set_fan(&self, value: f64) -> Result<f64, Box<dyn Error>> {
        let val = value.to_string();
        let resp = self.query(&[("FanSet", Some(val.as_str()))]).await?;
        Ok(resp["FanSet"].as_f64().unwrap())
    }
}
