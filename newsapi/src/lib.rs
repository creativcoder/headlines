#[cfg(feature = "async")]
use reqwest::Method;
use serde::Deserialize;
use url::Url;

const BASE_URL: &str = "https://newsapi.org/v2";

#[derive(thiserror::Error, Debug)]
pub enum NewsApiError {
    #[error("Failed fetching articles")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed converting response to string")]
    FailedResponseToString(#[from] std::io::Error),
    #[error("Article Parsing failed")]
    ArticleParseFailed(#[from] serde_json::Error),
    #[error("Url parsing failed")]
    UrlParsing(#[from] url::ParseError),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
    #[error("Async request failed")]
    #[cfg(feature = "async")]
    AsyncRequestFailed(#[from] reqwest::Error)
}

#[derive(Deserialize, Debug)]
pub struct NewsAPIResponse {
    status: String,
    articles: Vec<Article>,
    code: Option<String>,
}

impl NewsAPIResponse {
    pub fn articles(&self) -> &Vec<Article> {
        &self.articles
    }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    title: String,
    url: String,
    description: Option<String>
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn desc(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

pub enum Endpoint {
    TopHeadlines,
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::TopHeadlines => "top-headlines".to_string(),
        }
    }
}

pub enum Country {
    Us,
}

impl ToString for Country {
    fn to_string(&self) -> String {
        match self {
            Self::Us => "us".to_string(),
        }
    }
}

pub struct NewsAPI {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
}

impl NewsAPI {
    pub fn new(api_key: &str) -> NewsAPI {
        NewsAPI {
            api_key: api_key.to_string(),
            endpoint: Endpoint::TopHeadlines,
            country: Country::Us,
        }
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut NewsAPI {
        self.endpoint = endpoint;
        self
    }

    pub fn country(&mut self, country: Country) -> &mut NewsAPI {
        self.country = country;
        self
    }

    fn prepare_url(&self) -> Result<String, NewsApiError> {
        let mut url = Url::parse(BASE_URL)?;
        url.path_segments_mut()
            .unwrap()
            .push(&self.endpoint.to_string());

        let country = format!("country={}", self.country.to_string());
        url.set_query(Some(&country));

        Ok(url.to_string())
    }

    pub fn fetch(&self) -> Result<NewsAPIResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let req = ureq::get(&url).set("Authorization", &self.api_key);
        let response: NewsAPIResponse = req.call()?.into_json()?;
        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_response_err(response.code)),
        }
    }

    #[cfg(feature = "async")]
    pub async fn fetch_async(&self) -> Result<NewsAPIResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let client = reqwest::Client::new();
        let request = client
            .request(Method::GET, url)
            .header("Authorization", &self.api_key)
            .build()
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        let response: NewsAPIResponse = client
            .execute(request)
            .await?
            .json()
            .await
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_response_err(response.code)),
        }
    }
}

fn map_response_err(code: Option<String>) -> NewsApiError {
    if let Some(code) = code {
        match code.as_str() {
            "apiKeyDisabled" => NewsApiError::BadRequest("Your API key has been disabled"),
            _ => NewsApiError::BadRequest("Unknown error"),
        }
    } else {
        NewsApiError::BadRequest("Unknown error")
    }
}
