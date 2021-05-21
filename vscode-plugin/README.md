# vscode-fume README

### When does the vscode-plugin get activated?
Currently it gets activated once you open a file with ".fm" extension.
### Testing as a real installed Extension
* To start using your extension with Visual Studio Code copy vscode-plugin into the `<user home>/.vscode/extensions` folder and restart Code.
* Copy /fume-server folder as well in order that the vscode-plugin can start the LSP Server once it is activated.

### Testing in Debug mode
* In order to start the Debug mode, open `vscode-plugin` in VSCode, make sure that it's opened as root/main workspace - in order to avoid any problems.
* Make sure that in `Run and Debug` Tab that "Launch Client" is selected - press F5 and new VSCode Debug Window will be opened.
* Within that Window open a .fm file like "main.fm" - which will activate Fume-server, which currently needs to be in the same root folder as vscode-plugin.

### Testing in Debug mode with the attached Server
* (This is only needed if you are developing the Server.)
* Install this extension -> [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
* Repeat the steps outlined in "Testing in Debug mode", then go back `Run and Debug` Tab, from the dropdown menu
choose "Fume Server" which will attach the server in the debug mode as well.
