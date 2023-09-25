# Features to Add

* conversation metadata: The model is available back from the response, .  The temperature must be kept.  This metadata should be per response, and be displayed in the response display.  A time?
* Fonts: I am not in control of the fonts
* Need to have the initial purpose prompt
* Implement temperature
* Recover from invalid session inside chat_div
* Replay a conversation (starting a new conversation?)
* Popup/tool-tip type thing
* I need a way to enter whole files.  Copy and paste will do at first.

# BUG(s)

* FAILED Borrows!!  Making a new converstaion triggers it.  Added panics on each failure 
* `update_response_screen` Needs error checking
* The text in the conversation names is displed in the side panel from the prompt,  It needs to be sanitises.  The string "<input>" is interpreted as HTML
