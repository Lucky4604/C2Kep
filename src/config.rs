use base64;

pub struct Config {  
    pub websocket_base: &'static str,
    pub opcua_url: &'static str,
    pub username: &'static str,
    pub password: &'static str,
}

pub const PRODUCTION: Config = Config { 
    websocket_base: "wss://qa65.assetsense.com/ws/proto",
    opcua_url: "opc.tcp://192.168.1.7:49320", 
    username: "saikiran.o2", 
    password: "HydeVil#71",
};

pub fn get_config() -> &'static Config { 
    &PRODUCTION 
}

pub fn get_encoded_websocket_url() -> String {
    let config = get_config();
    let credentials = format!("{}:{}", config.username, config.password);
    let encoded_credentials = base64::encode(credentials.as_bytes());
    
    format!(
        "{}/{}/true",
        config.websocket_base, 
        encoded_credentials
    )
}

pub fn get_authorization_header() -> String {
    let config = get_config();
    let credentials = format!("{}:{}", config.username, config.password);
    format!("Basic {}", base64::encode(credentials.as_bytes()))
}