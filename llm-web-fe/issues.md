# Features to Add

* conversation metadata: The model is available back from the response, .  The temperature must be kept.  This metadata should be per response, and be displayed in the response display.  A time?
* Fonts: I am not in control of the fonts
* Need to have the initial purpose prompt
* Implement temperature
* Replay a conversation (starting a new conversation?)
* Popup/tool-tip type thing
* Button to copy a response to the clipboard
* A button to copy a conversation to the clipboard

# BUG(s)

* FAILED Borrows!!  Making a new converstaion triggers it.  Added panics on each failure 
* `update_response_screen` Needs error checking
* Word wrapping in the Response screen is broken, wraps in wrong places
* The left hand panel, for metadata, in the response screen is too wide.  Should word wrap the model name
