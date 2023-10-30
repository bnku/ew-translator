# Simple translator for X11  
Translate text from everywhere!  
Select text in some application, press hotkey and receive translation to pop-up window.  
It use Google Translate under the hood. 

### Dependencies
`xsel` [(source)](https://github.com/kfish/xsel) needed in `PATH`

### Use
```sh 
$ ew-translator -l [target language] -h [hotkeys]
```  
e.g. `ew-translator -l ru -h ctrl+shift+f7`  
or `ew-translator -l fr -h z`

![](video.webm)

### Help
```sh
USAGE:
    ew-translator [OPTIONS]

OPTIONS:
    -h, --hotkeys <HOTKEYS>    Hotkeys (modifier+key). Modifiers: ALT, CTRL, SHIFT, SUPER; Keys: 0-
                               9, A-Z, F1-F12, BACKSPACE, TAB, ENTER, CAPS_LOCK, ESCAPE, SPACEBAR,
                               PAGE_UP, PAGE_DOWN, END, HOME, ARROW_LEFT, ARROW_RIGHT, ARROW_UP,
                               ARROW_DOWN, PRINT_SCREEN, INSERT, DELETE [default: CTRL+SHIFT+F7]
        --help                 Print help information
    -l, --lang <LANG>          Target language [default: ru]
    -V, --version              Print version information
 ```