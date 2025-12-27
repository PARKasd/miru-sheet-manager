use dotenv::dotenv;
use tokio;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct SheetResponse {
    values: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    private_key: String,
    client_email: String,
    token_uri: String,
}


#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: usize,
    iat: usize,
}
async fn read_google_sheet(client: &reqwest::Client) -> Result<SheetResponse, Box<dyn std::error::Error>> {
    let iat = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
    let exp = iat + 3600;
    let sa: ServiceAccountKey =
        serde_json::from_reader(std::fs::File::open("miru_cred.json")?)?;

    let claims = Claims {
        iss: &sa.client_email,
        scope: "https://www.googleapis.com/auth/spreadsheets.readonly",
        aud: &sa.token_uri,
        iat,
        exp,
    };

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(sa.private_key.as_bytes())?;

    let jwt = encode(&header, &claims, &encoding_key)?;

    // OAuth2 access token 요청
    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", &jwt),
    ];

    let res = client
        .post(&sa.token_uri)
        .form(&params)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let token = res["access_token"].as_str().unwrap().to_string();

    let spreadsheet_id = dotenv::var("SHEET_ID").unwrap();
    let range = "Form Responses 1!A1:Z600";

    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        spreadsheet_id,
        urlencoding::encode(range)
    );

    let response = client
        .get(&url)
        .bearer_auth(&token)   // <- 이게 핵심
        .send()
        .await?
        .json::<SheetResponse>()
        .await?;

    Ok(response)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = reqwest::Client::new();
    let added_number =  dotenv::var("ADDED_PHONE").unwrap();
    let target_number: Vec<String> = added_number.split(",").map(|x| x.to_string()).collect();
    let added_mails = dotenv::var("ADDED_MAIL").unwrap();
    let target_mails: Vec<String> = added_mails.split(",").map(|x| x.to_string()).collect();
    let mut invite_number: Vec<String> = Vec::new();
    let mut invite_mails: Vec<String> = Vec::new();
    let sheet_data = read_google_sheet(&client).await?;
    for i in &sheet_data.values {
        let mut new_vec: Vec<String> = Vec::new();
        new_vec.push(i[2].clone());
        new_vec.push(i[1].clone());
        if i[5].clone().find("-") == None && i[5].clone().find("전화번호") == None {
            let org_phone_num = i[5].clone();
            let new_phone_num: String = format!("{}-{}-{}",&org_phone_num.to_string()[0..3], &&org_phone_num.to_string()[3..7], &org_phone_num.to_string()[7..]);
            new_vec.push(new_phone_num);
        }
        else {new_vec.push(i[5].clone());}

        if target_number.contains(&new_vec[0].to_string()){
            invite_number.push(new_vec[2].to_string());
        }
        if !target_mails.contains(&new_vec[1].to_string()) && new_vec[1].clone().to_string().find("hanyang") != None {
            invite_mails.push(new_vec[1].to_string());
        }
    }
    println!("invite mails: {:?}", invite_mails.join(","));
    println!("invite_numbers: {:?}", invite_number.join(","));
    Ok(())
}