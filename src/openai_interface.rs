use crate::api_error::ApiError;
use crate::api_error::ApiErrorType;
use crate::api_result::ApiResult;
use crate::fine_tune::FineTune;
use crate::json::AudioTranscriptionResponse;
use crate::json::ChatRequestInfo;
use crate::json::CompletionRequestInfo;
use crate::json::FileDeletedResponse;
use crate::json::FileInfoResponse;
use crate::json::FileUploadResponse;
use crate::json::Files;
// use crate::json::FineTuneCreateResponse;
// use crate::json::FtRoot;
use crate::json::ImageRequestInfo;
use crate::json::Message;
use crate::json::ModelReturned;
use crate::json::Usage;
use chrono::{NaiveDateTime, TimeZone, Utc};
use curl::easy::Easy;
use curl::easy::List;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::blocking::RequestBuilder;
use reqwest::header::HeaderMap;
use reqwest::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io::Read;
use std::path::Path;
use std::result::Result;
use std::time::Instant;

// URLS:
// * => implemented
// * Completions: POST https://api.openai.com/v1/completions
// * Chat: POST https://api.openai.com/v1/completions
// Edits: POST https://api.openai.com/v1/chat/completions
// * Images, create: POST https://api.openai.com/v1/images/generations
// * Images, edit: POST https://api.openai.com/v1/images/edits
// Images, variations: POST https://api.openai.com/v1/images/variations
// * Audio, transcription: POST https://api.openai.com/v1/audio/transcriptions
// Audio, translation: POST https://api.openai.com/v1/audio/translations
// * Files, list: GET https://api.openai.com/v1/files
// * Files, upload: POST https://api.openai.com/v1/files
// * Files, delete: DELETE https://api.openai.com/v1/files/{file_id}
// * Files, retrieve: GET https://api.openai.com/v1/files/{file_id}
// * Files, retrieve content: GET https://api.openai.com/v1/files/{file_id}/content
// Fine tune, create: POST https://api.openai.com/v1/fine-tunes
// Fine tune, list: GET https://api.openai.com/v1/fine-tunes
// Fine tune, retrieve: GET https://api.openai.com/v1/fine-tunes/{fine_tune_id}
// Fine tune, cancel: POST https://api.openai.com/v1/fine-tunes/{fine_tune_id}/cancel
// Fine tune, events: GET https://api.openai.com/v1/fine-tunes/{fine_tune_id}/events
// Fine tune, delete: DELETE https://api.openai.com/v1/models/{model}
// Moderations: POST https://api.openai.com/v1/moderations

/// Bas URI for requests
const API_URL: &str = "https://api.openai.com/v1";

#[derive(Debug)]
pub struct ApiInterface<'a> {
    /// Handles the communications with OpenAI
    client: Client,

    /// The secret key from OpenAI
    api_key: &'a str,

    /// Restricts the amount of text returned
    pub tokens: u32,

    /// Influences the predictability/repeatability of the model
    pub temperature: f32,

    /// Chat keeps its state here.
    pub context: Vec<String>,

    /// The chat model system prompt
    pub system_prompt: String,
}

impl Display for ApiInterface<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Temperature: {}\n\
		     Tokens: {}\n\
		     Context length: {}\n\
		     System prompt: {}",
            self.temperature,
            self.tokens,
            self.context.len(),
            self.system_prompt,
        )
    }
}

impl<'a> ApiInterface<'_> {
    pub fn new(api_key: &'a str, tokens: u32, temperature: f32) -> ApiInterface<'a> {
        ApiInterface {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(1200))
                .pool_idle_timeout(None)
                .connection_verbose(false)
                .build()
                .unwrap(),
            api_key,
            tokens,
            temperature,
            // model: model.to_string(),
            context: vec![],
            system_prompt: String::new(),
        }
    }

    /// Get information about a file
    pub fn file_info(&self, file_id: String) -> Result<ApiResult<String>, Box<dyn Error>> {
        // GET https://api.openai.com/v1/files/{file_id}
        let uri = format!("{API_URL}/files/{file_id}");
        let response = self
            .client
            .get(uri.as_str())
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()?;
        let headers = Self::header_map_to_hash_map(response.headers());
        if response.status() != StatusCode::OK {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )))
        } else {
            let fir: FileInfoResponse = response.json()?;
            let datetime = NaiveDateTime::from_timestamp_opt(fir.created_at as i64, 0).unwrap();
            let datetime_utc = Utc.from_utc_datetime(&datetime);

            let datetime_string = datetime_utc.format("%Y-%m-%d %H:%M:%S").to_string();
            Ok(ApiResult {
                headers,
                body: format!(
                    "Size: {} Name: {} Created: {}",
                    fir.bytes, fir.filename, datetime_string
                ), //fir.to_string(),
            })
        }

        // //let result = ;

        // Ok(ApiResult::new("".to_string(), HashMap::new())) // { headers: (), body: ()
    }

    /// Get file cotents
    pub fn file_contents(&self, file_id: String) -> Result<ApiResult<String>, Box<dyn Error>> {
        // GET https://api.openai.com/v1/files/{file_id}/content
        let uri = format!("{API_URL}/files/{file_id}/content");
        let response = self
            .client
            .get(uri.as_str())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;
        let headers = Self::header_map_to_hash_map(response.headers());
        if response.status() != StatusCode::OK {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )))
        } else {
            let content = response.text()?;
            Ok(ApiResult::new(content, headers))
        }
    }
    /// Delete a file
    pub fn files_delete(&self, file_id: String) -> Result<ApiResult<()>, Box<dyn Error>> {
        // DELETE https://api.openai.com/v1/files/{file_id}
        let uri = format!("{API_URL}/files/{file_id}");
        let response = self
            .client
            .delete(uri.as_str())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;
        let headers = Self::header_map_to_hash_map(response.headers());
        if response.status() != StatusCode::OK {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )))
        } else {
            let fdr: FileDeletedResponse = response.json()?;
            if !fdr.deleted || fdr.object != *"file" || fdr.id != file_id {
                Err(Box::new(ApiError::new(
                    ApiErrorType::Error(format!(
                        "File delete response:{:?}  file_id: {file_id}",
                        fdr
                    )),
                    headers,
                )))
            } else {
                Ok(ApiResult::new_e(HashMap::new()))
            }
        }
    }
    /// Get a list of all files stored on OpenAI
    pub fn files_list(&self) -> Result<ApiResult<Vec<(String, String)>>, Box<dyn Error>> {
        // GET https://api.openai.com/v1/files
        let uri = format!("{}/files", API_URL);
        let response = self
            .client
            .get(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        let headers = Self::header_map_to_hash_map(response.headers());
        let response_strings: Vec<(String, String)> = if response.status() != StatusCode::OK {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            return Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )));
        } else {
            response
                .json::<Files>()?
                .data
                .iter()
                .map(|x| (x.filename.clone(), x.id.clone()))
                .collect()
        };
        Ok(ApiResult::new_v(response_strings, headers))
    }

    /// Upload a file for fine-tuning.
    pub fn files_upload_fine_tuning(
        &self,
        file: &Path,
    ) -> Result<ApiResult<String>, Box<dyn Error>> {
        // Request
        // curl https://api.openai.com/v1/files \
        // -H "Authorization: Bearer $OPENAI_API_KEY" \
        // -F purpose="fine-tune" \
        // -F file="@mydata.jsonl"

        // Response
        // {
        //   "id": "file-XjGxS3KTG0uNmNOK362iJua3",
        //   "object": "file",
        //   "bytes": 140,
        //   "created_at": 1613779121,
        //   "filename": "mydata.jsonl",
        //   "purpose": "fine-tune"
        // }

        let uri = format!("{}/files", API_URL);

        let file_field = multipart::Part::file(file)?;
        let purpose_field = multipart::Part::text("fine-tune");
        let form = multipart::Form::new()
            .part("file", file_field)
            .part("purpose", purpose_field);
        let response = self
            .client
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()?;
        let headers = Self::header_map_to_hash_map(response.headers());
        let response_text: String = if response.status() != StatusCode::OK {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            return Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )));
        } else {
            response.json::<FileUploadResponse>()?.id
        };

        Ok(ApiResult::new(response_text, headers))
    }
    /// The audio file `audio_file` is tracscribed.  No `Usage` data
    /// returned from this endpoint
    /// Get an audio transcription
    pub fn audio_transcription(
        &mut self,
        audio_file: &Path,
        prompt: Option<&str>,
    ) -> Result<ApiResult<String>, Box<dyn Error>> {
        // Request
        // curl https://api.openai.com/v1/audio/transcriptions \
        //   -H "Authorization: Bearer $OPENAI_API_KEY" \
        //   -H "Content-Type: multipart/form-data" \
        //   -F file="@/path/to/file/audio.mp3" \
        //   -F model="whisper-1"

        // Respponse
        // {
        //   "text": "Imagine the....that."
        // }

        let uri = format!("{}/audio/transcriptions", API_URL);

        let file_field = multipart::Part::file(audio_file)?;
        let model_field = multipart::Part::text("whisper-1");
        let mut form = multipart::Form::new()
            .part("file", file_field)
            .part("model", model_field);
        if let Some(prompt) = prompt {
            let p: String = prompt.to_string();
            let prompt_field = multipart::Part::text(p);
            form = form.part("prompt", prompt_field);
        }

        // let client = reqwest::blocking::Client::new();
        let response = self
            .client
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()?;

        let headers = Self::header_map_to_hash_map(response.headers());
        let response_text: String = if response.status() != StatusCode::OK {
            format!(
                "Failed: Status: {}.\nResponse.path({})",
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown Reason"),
                response.url().path(),
            )
        } else {
            response.json::<AudioTranscriptionResponse>()?.text
        };

        Ok(ApiResult::new(response_text, headers))
    }
    // pub fn fine_tune_retrieve(
    //     &self,
    //     id: String,
    // ) -> Result<ApiResult<FineTuneCreateResponse>, Box<dyn Error>> {
    //     let uri = format!("{API_URL}/fine-tunes/{id}");
    //     let response = self
    //         .client
    //         .get(uri)
    //         .header("Authorization", format!("Bearer {}", self.api_key))
    //         .send()?;

    //     let headers = Self::header_map_to_hash_map(response.headers());
    //     let body: FineTuneCreateResponse = if response.status() != StatusCode::OK {
    //         let reason = response
    //             .status()
    //             .canonical_reason()
    //             .unwrap_or("Unknown Reason");
    //         return Err(Box::new(ApiError::new(
    //             ApiErrorType::Status(response.status(), reason.to_string()),
    //             headers,
    //         )));
    //     } else {
    //         response.json::<FineTuneCreateResponse>()?
    //     };

    //     Ok(ApiResult { headers, body })
    // }
    pub fn fine_tune_create(
        &self,
        training_file_id: String,
    ) -> Result<ApiResult<FineTune>, Box<dyn Error>> {
        let uri = format!("{API_URL}/fine-tunes");
        let request_body = json!({
                "training_file": training_file_id.as_str()
        });

        let mut response = self
            .client
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .json(&request_body)
            .send()?;
        let mut s = String::new();
        _ = response.read_to_string(&mut s)?;
        let headers = Self::header_map_to_hash_map(response.headers());
        let st = s.as_str();
        let fine_tune: FineTune = serde_json::from_str(st)?;

        // let body: FineTune = if response.status() != StatusCode::OK {
        //     let reason = response
        //         .status()
        //         .canonical_reason()
        //         .unwrap_or("Unknown Reason");
        //     return Err(Box::new(ApiError::new(
        //         ApiErrorType::Status(response.status(), reason.to_string()),
        //         headers,
        //     )));
        // } else {
        //     response.json::<FineTune>()?
        // };

        Ok(ApiResult {
            headers,
            body: fine_tune,
        })
    }
    // pub fn fine_tune_info(
    //     &self,
    //     id: String,
    // ) -> Result<ApiResult<FineTuneCreateResponse>, Box<dyn Error>> {
    //     let uri = format!("{API_URL}/fine-tunes/{id}");
    //     let response = self
    //         .client
    //         .get(uri)
    //         .header("Authorization", format!("Bearer {}", self.api_key))
    //         .send()?;

    //     let headers = Self::header_map_to_hash_map(response.headers());
    //     let body: FineTuneCreateResponse = if response.status() != StatusCode::OK {
    //         let reason = response
    //             .status()
    //             .canonical_reason()
    //             .unwrap_or("Unknown Reason");
    //         return Err(Box::new(ApiError::new(
    //             ApiErrorType::Status(response.status(), reason.to_string()),
    //             headers,
    //         )));
    //     } else {
    //         response.json::<FineTuneCreateResponse>()?
    //     };

    //     Ok(ApiResult { headers, body })
    // }
    // pub fn fine_tunes_list(&self) -> Result<ApiResult<FtRoot>, Box<dyn Error>> {
    //     // endpoint
    //     let uri = format!("{API_URL}/fine-tunes");
    //     let response = self
    //         .client
    //         .get(uri)
    //         .header("Authorization", format!("Bearer {}", self.api_key))
    //         .send()?;

    //     let headers = Self::header_map_to_hash_map(response.headers());
    //     let response: FtRoot = if response.status() != StatusCode::OK {
    //         let reason = response
    //             .status()
    //             .canonical_reason()
    //             .unwrap_or("Unknown Reason");
    //         return Err(Box::new(ApiError::new(
    //             ApiErrorType::Status(response.status(), reason.to_string()),
    //             headers,
    //         )));
    //     } else {
    //         response.json::<FtRoot>()?
    //     };

    //     Ok(ApiResult {
    //         headers, // HashMap::new(),
    //         body: response,
    //     })
    // }
    /// Finetune
    /// Workflow:
    ///

    /// Documented [here](https://platform.openai.com/docs/api-reference/chat)
    pub fn chat(&mut self, prompt: &str, model: &str) -> Result<ApiResult<String>, Box<dyn Error>> {
        // An ongoing conversation with the LLM

        // endpoint
        let uri = format!("{}/chat/completions", API_URL);

        // Model can be any of: gpt-4, gpt-4-0314, gpt-4-32k,
        // gpt-4-32k-0314, gpt-3.5-turbo, gpt-3.5-turbo-0301
        // https://platform.openai.com/docs/models/model-endpoint-compatibility

        // Put the conversation so far in here
        let mut messages: Vec<Message> = vec![]; // = [Message { role, content }];

        // If here is any context, supply it
        if self.context.is_empty() {
            // Conversation starting.  Append system prompt to context
            messages.push(Message {
                role: "system".to_string(),
                content: self.system_prompt.clone(),
            });
        } else {
            for i in 0..self.context.len() {
                messages.push(Message {
                    role: "user".to_string(),
                    content: self.context[i].clone(),
                });
            }
        }

        // Add in the latest installment, the prompt for this function
        let role = "user".to_string();
        let content = prompt.to_string();
        messages.push(Message { role, content });

        // The payload
        let data = json!({
            "messages": messages,
            "model": model,
        });

        // Send the request and get the Json data as a String, convert
        // into ``ChatRequestInfo`
        let (headers, response_string) = self.send_curl(&data, uri.as_str())?;
        let json: ChatRequestInfo = serde_json::from_str(response_string.as_str())?;
        let mut headers_ret = Self::usage_headers(json.usage.clone());
        let cost: f64 = Self::cost(json.usage, model);
        // Define a function that returns a closure
        // fn update_spent(cost: f64) -> impl FnMut(SharedState) -> SharedState {
        //     move |mut ss| {
        //         ss.spent += cost;
        //         ss
        //     }
        // }

        // // Call read_write_atomic with the closure
        // let ss: SharedState = SharedState::read_write_atomic(update_spent(cost))?

        headers_ret.insert("Cost".to_string(), format!("{}", cost)); //ss.spent));

        headers_ret.extend(headers);

        let content = json.choices[0].message.content.clone();
        self.context.push(prompt.to_string());
        self.context.push(content.clone());

        Ok(ApiResult::new(content, headers_ret))
    }

    /// Read the record of the conversation
    pub fn get_context(&self) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(self.context.clone())
    }

    /// Restore a record of a conversation
    pub fn set_context(&mut self, context: Vec<String>) {
        self.context = context;
    }
    /// [Documented](https://platform.openai.com/docs/api-reference/completions)
    /// Takes the `prompt` and sends it to the LLM with no context.
    /// The interface has to manage no state
    pub fn completion(
        &mut self,
        prompt: &str,
        model: &str,
    ) -> Result<ApiResult<String>, Box<dyn Error>> {
        let uri: String = format!("{}/completions", API_URL);

        let payload = CompletionRequestInfo::new(prompt, model, self.temperature, self.tokens);

        let response = self
            .client
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?;

        let mut headers = Self::header_map_to_hash_map(response.headers());
        let response_text: String = if response.status() != StatusCode::OK {
            // There was some sort of failure.  Probably a network
            // failure
            format!(
                "Failed: Status: {}.\nResponse.path({})",
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown Reason"),
                response.url().path(),
            )
        } else {
            // Got a good response from the LLM
            let response_debug = format!("{:?}", &response);
            let json: CompletionRequestInfo = match response.json() {
                Ok(json) => json,
                Err(err) => {
                    panic!("Failed to get json.  {err}\n{response_debug}")
                }
            };

            // The data about the query
            // let choice_count = json.choices.len();
            let finish_reason = json.choices[0].finish_reason.as_str();
            if finish_reason != "stop" {
                headers.insert("finsh reason".to_string(), finish_reason.to_string());
            }

            if json.choices[0].text.is_empty() {
                panic!("Empty json.choices[0].  {:?}", &json);
            } else {
                json.choices[0].text.clone()
            }
        };
        Ok(ApiResult::new(response_text, headers))
    }

    /// Handle image mode prompts
    pub fn image(&mut self, prompt: &str) -> Result<ApiResult<String>, Box<dyn Error>> {
        // Endpoint
        let uri: String = format!("{}/images/generations", API_URL);

        // Payload
        let data = json!({
                  "prompt":  prompt,
                  "size": "1024x1024",
        });

        // Set up network comms
        let res = Client::new()
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key).as_str())
            .header("Content-Type", "application/json")
            .json(&data);

        // Send network request
        let response = match res.send() {
            Ok(r) => r,
            Err(err) => {
                return Ok(ApiResult::new(
                    format!("Image: Response::send() failed: '{err}'"),
                    HashMap::new(),
                ));
            }
        };

        // Prepare diagnostic data
        let headers = Self::header_map_to_hash_map(&response.headers().clone());
        if !response.status().is_success() {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            return Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )));
            //return Ok(ApiResult::new("Request failed".to_string(), headers));
        }

        // Have a normal result.  Process it
        let json: ImageRequestInfo = match response.json() {
            Ok(json) => json,
            Err(err) => {
                return Err(Box::new(ApiError::new(
                    ApiErrorType::BadJson(format!("{err}")),
                    headers,
                )))
            }
        };

        // Success.
        Ok(ApiResult::new(json.data[0].url.clone(), headers))
    }

    // Editing an image.  The mask defines the region to edit
    // according to the prompt.  ??The prompt describes the whole
    // image??
    // https://platform.openai.com/docs/api-reference/images/create-edit
    pub fn image_edit(
        &mut self,
        prompt: &str,
        image: &Path,
        mask: &Path,
    ) -> Result<ApiResult<String>, Box<dyn Error>> {
        // Endpoint
        let uri = format!("{}/images/edits", API_URL);

        // Some timeing.  TODO: Why here, in this function, and not everywhere?
        let start = Instant::now();

        // Need an image to edit.  If there is an image in `self.image`
        // prefer that.  Failing that use `self.focus_image_url` In the
        // second case the image refered to in the url is downloaded and
        // put into `self.image`

        // let mask_path = mask_file.path().to_owned();

        // Prepare the payload to send to OpenAI
        let form = multipart::Form::new();
        let form = match form.file("image", image) {
            Ok(f) => match f.file("mask", mask) {
                Ok(s) => s
                    .text("prompt", prompt.to_string())
                    .text("size", "1024x1024"),
                Err(err) => {
                    return Err(Box::new(ApiError::new(
                        ApiErrorType::Error(format!("{err}")),
                        HashMap::new(),
                    )))
                }
            },
            // Err(err) => return Err(Box::new(ApiErrorType::Error("Err path: {err}".to_string()))),
            Err(err) => {
                return Err(Box::new(ApiError::new(
                    ApiErrorType::Error(format!("{err}")),
                    HashMap::new(),
                )))
            }
        };

        // Set up network comms
        let req_build: RequestBuilder = Client::new()
            .post(uri.as_str())
            .timeout(std::time::Duration::from_secs(1200))
            .header("Authorization", format!("Bearer {}", self.api_key).as_str())
            .multipart(form);

        // Send request
        let response = match req_build.send() {
            Ok(r) => r,
            Err(err) => {
                println!("Failed url: {uri} Err: {err}");
                return Err(Box::new(err));
            }
        };

        let headers = Self::header_map_to_hash_map(&response.headers().clone());
        println!("Sent message: {:?}", start.elapsed());
        if !response.status().is_success() {
            let reason = response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown Reason");
            return Err(Box::new(ApiError::new(
                ApiErrorType::Status(response.status(), reason.to_string()),
                headers,
            )));
        }
        let response_dbg = format!("{:?}", response);
        // let response_text = response.text()?;
        // Ok(response_text)
        let json: ImageRequestInfo = match response.json() {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Failed to get json. {err} Response: {response_dbg}");
                return Err(Box::new(err));
            }
        };

        Ok(ApiResult::new(json.data[0].url.clone(), headers))
    }

    /// Handle the response if the user queries what models there are
    /// ("! md" prompt in cli).  
    pub fn model_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let uri: String = format!("{}/models", API_URL);
        let response = self
            .client
            .get(uri.as_str())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;
        if !response.status().is_success() {
            // If it were not a success the previous cal will have failed
            // This will not happen
            panic!("Failed call to get model list. {:?}", response);
        }
        let model_returned: ModelReturned = response.json().unwrap();
        println!("{:?}", model_returned);
        Ok(model_returned.data.iter().map(|x| x.root.clone()).collect())
        // Ok(vec![])
    }

    /// Convert the usege into a price.
    fn cost(usage: Usage, model: &str) -> f64 {
        // GPT-4is more expensive
        if model.starts_with("gpt-4") {
            usage.completion_tokens as f64 / 1000.0 * 12.0
                + usage.prompt_tokens as f64 / 1000.0 * 0.06
        } else if model.starts_with("gpt-3") {
            usage.total_tokens as f64 / 1000.0 * 0.2
        } else {
            panic!("{model}");
        }
    }
    fn usage_headers(usage: Usage) -> HashMap<String, String> {
        let prompt_tokens = usage.prompt_tokens.to_string();
        let completion_tokens = usage.completion_tokens.to_string();
        let total_tokens = usage.total_tokens.to_string();
        let mut result = HashMap::new();
        result.insert("Tokens prompt".to_string(), prompt_tokens);
        result.insert("Tokens completion".to_string(), completion_tokens);
        result.insert("Tokens total".to_string(), total_tokens);
        result
    }

    /// Used to adapt headers reported from Reqwest
    fn header_map_to_hash_map(header_map: &HeaderMap) -> HashMap<String, String> {
        let mut hash_map = HashMap::new();
        for (header_name, header_value) in header_map.iter() {
            if let (Ok(name), Ok(value)) = (
                header_name.to_string().as_str().trim().parse::<String>(),
                header_value.to_str().map(str::to_owned),
            ) {
                hash_map.insert(name, value);
            }
        }
        hash_map
    }

    /// Clear the context used to maintain chat history
    pub fn clear_context(&mut self) {
        self.context.clear();
    }

    /// Send a request, the body of which is coded in `data`, to `uri`.
    /// Return the Json data as a String
    fn send_curl(
        &mut self,
        data: &serde_json::Value,
        uri: &str,
    ) -> Result<(HashMap<String, String>, String), Box<dyn Error>> {
        let body = format!("{data}");

        let mut body = body.as_bytes();
        let mut curl_easy = Easy::new();
        curl_easy.url(uri)?;

        // Prepare the headers
        let mut list = List::new();
        list.append(format!("Authorization: Bearer {}", self.api_key).as_str())?;
        list.append("Content-Type: application/json")?;
        curl_easy.http_headers(list)?;

        // I am unsure why I have to do this magick incantation
        curl_easy.post_field_size(body.len() as u64)?;

        // To get the normal output of the server
        let mut output_buffer = Vec::new();

        // To get the headers
        let mut header_buffer = Vec::new();

        // Time the process.
        let start = Instant::now();

        {
            // Start a block so `transfer` is destroyed and releases the
            // borrow it has on `header_buffer` and `output_buffer`
            let mut transfer = curl_easy.transfer();
            transfer.header_function(|data| {
                header_buffer.push(String::from_utf8(data.to_vec()).unwrap());
                true
            })?;
            transfer.read_function(|buf| Ok(body.read(buf).unwrap_or(0)))?;
            transfer.write_function(|data| {
                output_buffer.extend_from_slice(data);
                Ok(data.len())
            })?;
            transfer.perform()?;
        }

        // Made the call, got the output,  Close the timer
        let _duration = start.elapsed();

        let result = String::from_utf8(output_buffer)?; // Process the output

        let headers_hm: HashMap<String, String> = header_buffer
            .into_iter()
            .filter_map(|item| {
                let mut parts = item.splitn(2, ':');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    Some((key.to_string(), value.trim().to_string()))
                } else {
                    None
                }
            })
            .collect();
        Ok((headers_hm, result))
    }
}
