# Features to Add

* conversation metadata: Requires modification to ChatResponse to send data like model and temperature back.  The model is available back from the response, which makes sense because the model asked for is a sbset of the model used: ask for "gpt-3.5-turbo" and "gpt-3.5-turbo-0613" will be used.  The temperature must be kept.  Also the cost (listed in bugs) per conversation must be displayed
* Implement temperature
* Recover from invalid session inside chat_div
* Replay a conversation (starting a new conversation?)
* Popup/tool-tip type thing

# BUG(s)

* Now there are multiple conversations the cost per conversation should be in the side panel.  Currently only clear conversation cost when "clear conversation" is used.
* "clear conversations" button is deprecated
