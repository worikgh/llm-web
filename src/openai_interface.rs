use crate::json::AudioTranscriptionResponse;
use crate::json::ChatRequestInfo;
use crate::json::CompletionRequestInfo;
use crate::json::ImageRequestInfo;
use crate::json::Message;
use crate::json::ModelReturned;
use curl::easy::Easy;
use curl::easy::List;
use image::{ImageFormat, Rgba, RgbaImage};
// use std::fs::File;
// use std::io::BufReader;
use std::io::Write;
// use multipart::client::lazy::Multipart;
use reqwest::blocking::get;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::blocking::RequestBuilder;
use serde_json::{from_str, json, Value};
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
use std::time::Instant;

use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::result::Result;

// URLS:
// Completions: POST https://api.openai.com/v1/completions
// Chat: POST https://api.openai.com/v1/completions
// Edits: POST https://api.openai.com/v1/chat/completions
// Images, create: POST https://api.openai.com/v1/images/generations
// Images, edit: POST https://api.openai.com/v1/images/edits
// Images, variations: POST https://api.openai.com/v1/images/variations
// Audio, transcription: POST https://api.openai.com/v1/audio/transcriptions
// Audio, translation: POST https://api.openai.com/v1/audio/translations
// Files, list: GET https://api.openai.com/v1/files
// Files, upload: POST https://api.openai.com/v1/files
// Files, delete: DELETE https://api.openai.com/v1/files/{file_id}
// Files, retrieve: GET https://api.openai.com/v1/files/{file_id}
// Files, retrieve content: GET https://api.openai.com/v1/files/{file_id}/content
// Fine tune, create: POST https://api.openai.com/v1/fine-tunes
// Fine tune, list: GET https://api.openai.com/v1/fine-tunes
// Fine tune, retrieve: GET https://api.openai.com/v1/fine-tunes/{fine_tune_id}
// Fine tune, cancel: POST https://api.openai.com/v1/fine-tunes/{fine_tune_id}/cancel
// Fine tune, events: GET https://api.openai.com/v1/fine-tunes/{fine_tune_id}/events
// Fine tune, delete: DELETE https://api.openai.com/v1/models/{model}
// Moderations: POST https://api.openai.com/v1/moderations

/// Bas URI for requests
const API_URL: &str = "https://api.openai.com/v1";

/// The modes for the interaction with Open AI
#[derive(Debug, PartialEq)]
pub enum ModelMode {
    Completions,
    Chat,
    Image,
    ImageEdit,
}
impl std::fmt::Display for ModelMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ModelMode::Completions => "completions",
            ModelMode::Chat => "chat",
            ModelMode::Image => "image",
            ModelMode::ImageEdit => "image_edit",
        };
        write!(f, "{str}")
    }
}

#[derive(Debug)]
pub struct ModelModeParseErr;

impl std::str::FromStr for ModelMode {
    type Err = ModelModeParseErr;
    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        match mode {
            "completions" => Ok(ModelMode::Completions),
            "chat" => Ok(ModelMode::Chat),
            "image" => Ok(ModelMode::Image),
            "image_edit" => Ok(ModelMode::ImageEdit),
            _ => Err(ModelModeParseErr),
        }
    }
}

#[derive(Debug)]
pub struct ApiInterface<'a> {
    /// Handles the communications with OpenAI
    /// TODO: Replace this reqwest::blocking::Client with calls to Curl
    client: Client,

    /// The secret key from OpenAI
    api_key: &'a str,

    /// Restricts the amount of text returned
    pub tokens: u32,

    /// Influences the predictability/repeatability of the model
    pub temperature: f32,

    /// The model in use
    pub model: String,

    /// The mode of use
    pub model_mode: ModelMode,

    /// Chat keeps its state here.
    pub context: Vec<String>,

    /// The chat model system prompt
    pub system_prompt: String,

    /// The image model URL for the image that we are paying attention
    /// to.  Openai generated images
    pub focus_image_url: Option<String>,

    /// Image to use with image_edit mode.  User supplied or copied
    /// from `focus_image_url`
    pub image: Option<PathBuf>,

    /// Mask to use with image_edit mode.
    pub mask: Option<PathBuf>,
}

impl Display for ApiInterface<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mode: {:?}\n\
		     Temperature: {}\n\
		     Model: {}\n\
		     Tokens: {}\n\
		     Context length: {}\n\
		     System prompt: {}\n\
		     Image focus URI Set: {}\n\
		     Mask: {:?}\n",
            self.model_mode,
            self.temperature,
            self.model,
            self.tokens,
            self.context.len(),
            self.system_prompt,
            self.focus_image_url.is_some(),
            match &self.mask {
                Some(pb) => pb.display().to_string(),
                None => "<None>".to_string(),
            },
        )
    }
}

impl<'a> ApiInterface<'_> {
    pub fn new(
        api_key: &'a str,
        tokens: u32,
        temperature: f32,
        model: &'a str,
        model_mode: ModelMode,
    ) -> ApiInterface<'a> {
        ApiInterface {
            client: ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(1200))
                .pool_idle_timeout(None)
                .connection_verbose(true)
                .build()
                .unwrap(),
            api_key,
            tokens,
            temperature,
            model: model.to_string(),
            model_mode,
            context: vec![],
            system_prompt: String::new(),
            focus_image_url: None,
            mask: None,
            image: None,
        }
    }

    pub fn send_prompt(&mut self, prompt: &str) -> Result<String, Box<dyn Error>> {
        match self.model_mode {
            ModelMode::Completions => self.completion(prompt),
            ModelMode::Chat => self.chat(prompt),
            ModelMode::Image => self.image(prompt),
            ModelMode::ImageEdit => self.image_edit(prompt),
        }
    }

    /// Handle image mode prompts
    fn image(&mut self, prompt: &str) -> Result<String, Box<dyn Error>> {
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
                return Ok(format!("Image: Response::send() failed: '{err}'"));
            }
        };

        // Prepare diagnostic data
        let headers = response.headers().clone();
        let diagnostics = format!(
            "{}\n{}",
            self.after_request(Self::header_map_to_hash_map(&headers), None, "",)?,
            format!("{:?}", response),
        );

        if !response.status().is_success() {
            return Ok(format!("Request failed: {diagnostics}"));
        }

        // Have a normal result.  Process it
        let json: ImageRequestInfo = match response.json() {
            Ok(json) => json,
            Err(err) => {
                return Ok(format!(
                    "Failed to get json. {err} Diagnostics: {diagnostics}"
                ));
            }
        };

        // Display the image for the user.
        open_url(json.data[0].url.as_str())?;

        // Store the link to the image for refinement
        self.focus_image_url = Some(json.data[0].url.clone());

        // Success.
        Ok(format!("Opening: {}", json.data[0].url.clone()))
    }

    // Editing an image
    fn image_edit(&mut self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // Endpoint
        let uri = format!("{}/images/edits", API_URL);

        // Some timeing.  TODO: Why here, in this function, and not everywhere?
        let start = Instant::now();

        // Need an image to edit.  If there is an image in `self.image`
        // prefer that.  Failing that use `self.focus_image_url` In the
        // second case the image refered to in the url is downloaded and
        // put into `self.image`
        match (&self.focus_image_url, &self.image) {
            (Some(url), None) => {
                // Get the image to edit from the URL and put it in self.image
                println!("Get the URL");
                let mut img_data: Vec<u8> = Vec::new();
                get(url).unwrap().read_to_end(&mut img_data).unwrap();

                let mut incomming_image_file =
                    tempfile::Builder::new().suffix(".png").tempfile()?;
                incomming_image_file.write_all(&img_data)?;
                println!("Wrote image: {:?}", start.elapsed());

                // Must convert the image
                // convert otter.png -type TrueColor -define png:color-type=6 otter_rgba.png
                let img = image::open(incomming_image_file.path())?;
                // println!("3");

                // Ensure the image has an alpha channel
                let img_rgba = img.into_rgba8();

                // Save the image with PNG format and an alpha channel (color-type 6)
                let adjusted_image_file = tempfile::Builder::new().suffix(".png").tempfile()?;
                self.image = Some(adjusted_image_file.path().to_owned());
                adjusted_image_file.close()?;
                // let output_path = adjusted_image_path.into_os_string();
                // println!("Save converted file: {:?}", output_path);
                img_rgba.save_with_format(self.image.clone().unwrap(), ImageFormat::Png)?;
            }
            (_, Some(_)) => (),
            (None, None) => {
                panic!("No image to work with")
            }
        };

        // At this point the (path to the) image to edit is at `self.image`

        // Must have a mask.  If none defined use a 1024x1024 mask
        // (which is not much use...)
        let mask_path = match self.mask.clone() {
            Some(f) => f,
            None => {
                // A 1024x1024 transparent PNG image to use as a mask
                // Image dimensions
                let width = 1024;
                let height = 1024;
                let mask_path = tempfile::Builder::new()
                    .suffix(".png")
                    .tempfile()?
                    .path()
                    .to_path_buf();
                // Create the transparent RGBA image
                let transparent_color = Rgba([0, 0, 0, 0]);
                let img = RgbaImage::from_pixel(width, height, transparent_color);
                img.save(mask_path.clone()).unwrap();
                mask_path
            }
        };
        // let mask_path = mask_file.path().to_owned();

        // Prepare the payload to send to OpenAI
        let form = multipart::Form::new();
        let form = match form.file("image", self.image.clone().unwrap().as_path()) {
            Ok(f) => match f.file("mask", mask_path.clone()) {
                Ok(s) => s
                    .text("prompt", prompt.to_string())
                    .text("size", "1024x1024"),
                Err(err) => {
                    panic!("{:?}:{}: {err}", mask_path, mask_path.exists())
                }
            },
            Err(err) => {
                panic!("Err path: {err}")
            }
        };

        // Set up network comms
        let req_build: RequestBuilder = Client::new()
            .post(uri.as_str())
            .timeout(std::time::Duration::from_secs(1200))
            .header("Authorization", format!("Bearer {}", self.api_key).as_str())
            // .bearer_auth(bearer.as_str())
            .multipart(form);

        // Send request
        let response = match req_build.send() {
            Ok(r) => r,
            Err(err) => {
                println!("Failed url: {uri} Err: {err}");
                return Err(Box::new(err));
            }
        };

        let headers = response.headers().clone();
        let ar = self.after_request(Self::header_map_to_hash_map(&headers), None, "")?;
        println!("Sent message: {:?}", start.elapsed());
        if !response.status().is_success() {
            let json_str = response.text()?;
            let parsed_json: Result<Value, _> = from_str(json_str.as_str());
            let fmt = match parsed_json {
                Ok(j) => j,
                Err(err) => panic!("{err}"),
            };
            return Ok(format!(
                "{ar}Request failed: {}.",
                fmt // match  {
                    //   Ok(j) => j,
                    //   Err(err) => format!("{err}: Cannot format: {:?}", j),
                    // }
            ));
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
        open_url(json.data[0].url.as_str())?;
        self.focus_image_url = Some(json.data[0].url.clone());
        // mask_file.close().unwrap();

        Ok(format!("{ar}Opening: {}", json.data[0].url.clone()))
    }
    /// Documented [here](https://platform.openai.com/docs/api-reference/chat)
    fn chat(&mut self, prompt: &str) -> Result<String, Box<dyn Error>> {
        // An ongoing conversation with the LLM

        // endpoint
        let uri = format!("{}/chat/completions", API_URL);

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
            "model": self.model,
        });

        // Send the request and get the Json data as a String, convert
        // into ``ChatRequestInfo`
        let response_string = self.send_curl(&data, uri.as_str())?;
        let json: ChatRequestInfo = serde_json::from_str(response_string.as_str())?;
        let ar = self.after_request(HashMap::new(), Some(json.usage), "")?;
        let content = json.choices[0].message.content.clone();
        self.context.push(prompt.to_string());
        self.context.push(content.clone());

        Ok(format!("{ar}\n{content}"))
    }

    /// [Documented](https://platform.openai.com/docs/api-reference/completions)
    /// Takes the `prompt` and sends it to the LLM with no context.
    /// The interface has to manage no state
    fn completion(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let payload =
            CompletionRequestInfo::new(prompt, self.model.as_str(), self.temperature, self.tokens);
        let uri: String = format!("{}/completions", API_URL);

        let response = self
            .client
            .post(uri.as_str())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()?;
        let response_text: String;
        if response.status() != StatusCode::OK {
            // There was some sort of failure.  Probably a network
            // failure
            response_text = format!(
                "Failed: Status: {}.\nResponse.path({})",
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown Reason"),
                response.url().path(),
            );
        } else {
            // Got a good response from the LLM
            let response_debug = format!("{:?}", &response);
            let headers = response.headers().clone();
            let json: CompletionRequestInfo = match response.json() {
                Ok(json) => json,
                Err(err) => {
                    panic!("Failed to get json.  {err}\n{response_debug}")
                }
            };

            // The data about the query
            // let choice_count = json.choices.len();
            let finish_reason = json.choices[0].finish_reason.as_str();
            let extra = if finish_reason != "stop" {
                "Reason {finish_reason}"
            } else {
                ""
            };

            if json.choices[0].text.is_empty() {
                panic!("Empty json.choices[0].  {:?}", &json);
            } else {
                format!(
                    "{}\n{}",
                    self.after_request(
                        Self::header_map_to_hash_map(&headers),
                        Some(json.usage.clone()),
                        extra,
                    )?,
                    json.choices[0].text.clone()
                )
            }
        };
        Ok(response_text)
    }

    /// The audio file `audio_file` is tracscribed.  No `Usage` data
    /// returned from this endpoint
    pub fn audio_transcription(&mut self, audio_file: &Path) -> Result<String, Box<dyn Error>> {
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
        let form = multipart::Form::new()
            .part("file", file_field)
            .part("model", model_field);

        // let client = reqwest::blocking::Client::new();
        let response = self
            .client
            .post(uri)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()?;

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
            let headers = response.headers().clone();
            let json: AudioTranscriptionResponse = response.json()?;
            format!(
                "{}\n{}",
                self.after_request(Self::header_map_to_hash_map(&headers), None, "")?,
                json.text
            )
        };

        Ok(response_text)
    }

    /// Handle the response if the user queries what models there are
    /// ("! md" prompt in cli)
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

    /// Data about the request before it goes out
    fn after_request(
        &self,
        response_headers: HashMap<String, String>,
        usage: Option<crate::json::Usage>,
        extra: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = extra.to_string();
        let none_err = "<NONE>".to_string();

        let hn = "openai-model";
        let hv = response_headers.get(hn).unwrap_or(&none_err);
        result += format!("{hn}: '{hv}'\n",).as_str();

        let hn = "x-ratelimit-limit-requests";
        let hv = response_headers.get(hn).unwrap_or(&none_err);

        result += match hv.parse::<u32>() {
            Ok(_) => format!("{hn}: {hv}\n",),
            Err(err) => {
                format!("Cannot parse header: Name({hn}). Error({err}).  Value({hv})\n",)
            }
        }
        .as_str();

        let hn = "x-ratelimit-remaining-requests";
        let hv = response_headers.get(hn).unwrap_or(&none_err);
        result += match hv.parse::<u32>() {
            Err(err) => {
                format!("Cannot parse header: Name({hn}). Error({err}).  Value({hv})\n",)
            }

            Ok(_) => format!("{hn}: {hv}\n",),
        }
        .as_str();

        let hn = "x-ratelimit-reset-requests";
        let hv = response_headers.get(hn).unwrap_or(&none_err);
        result += format!("{hn}: {hv}\n",).as_str();
        if let Some(usage) = usage {
            let prompt_tokens = usage.prompt_tokens;
            let completion_tokens = usage.completion_tokens;
            let total_tokens = usage.total_tokens;
            result = format!(
                "{result} Tokens: {prompt_tokens} + {completion_tokens} \
		     == {total_tokens}\n"
            );
        }
        Ok(result)
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
    fn send_curl(&mut self, data: &serde_json::Value, uri: &str) -> Result<String, Box<dyn Error>> {
        let body = format!("{data}");

        let mut body = body.as_bytes();
        let mut curl_easy = Easy::new();
        curl_easy.url(uri)?;

        // Prepare the headers
        let mut list = List::new();
        list.append(format!("Authorization: Bearer {}", self.api_key).as_str())?;
        list.append("Content-Type: application/json")?;
        curl_easy.http_headers(list)?;

        // Set type of request
        curl_easy.post(true)?;

        // I am unsure why I have to do this magick incantation
        curl_easy.post_field_size(body.len() as u64)?;

        // Time the process.
        let start = Instant::now();

        // To get the normal output of the server
        let mut output_buffer = Vec::new();

        // To get the headers
        let mut header_buffer = Vec::new();

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
        let duration = start.elapsed();

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
        print!(
            "{}",
            self.after_request(
                headers_hm,
                None,
                format!("Query Duration: {:?}\n", duration).as_str(), //extra.as_str(),
            )?
        );
        Ok(result)
    }
}

/// Used to display the image that OpenAI generates.
fn open_url(url: &str) -> Result<(), Box<dyn Error>> {
    match webbrowser::open(url) {
        Ok(_) => {
            eprintln!("Opened in browser");
            Ok(())
        }
        Err(err) => Err(Box::new(err)),
    }
}
