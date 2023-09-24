# Features to Add

* conversation metadata: Requires modification to ChatResponse to send data like model and temperature back.  The model is available back from the response, which makes sense because the model asked for is a sbset of the model used: ask for "gpt-3.5-turbo" and "gpt-3.5-turbo-0613" will be used.  The temperature must be kept.  This metadata should be per response, and be displayed in the response display
* Implement temperature
* Recover from invalid session inside chat_div
* Replay a conversation (starting a new conversation?)
* Popup/tool-tip type thing

# BUG(s)

* "clear conversations" button is deprecated
