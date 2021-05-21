import { workspace, ExtensionContext, ExtensionMode } from 'vscode'

import {
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node'

let client: LanguageClient

export function activate(context: ExtensionContext) {
	client = new LanguageClient(
		'fume-server',
		'Fume',
		getServerOptions(context),
		getClientOptions()
	)

	// Start the client. This will also launch the server
	console.log("Starting Client and Server")
	client.start()
	
	client.onReady().then(_ => {
		console.log("Client has Connected to the Server successfully!")
	})
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined
	}
	return client.stop()
}


function getServerOptions(context: ExtensionContext): ServerOptions {
	const serverPath = context.asAbsolutePath('../fume-server')

	const serverExecutable: Executable = {
		command: 'cargo run',
		options: {
			cwd: serverPath,
			shell: true,
		}
	}

	const serverOptions: ServerOptions = {
		run: serverExecutable,
		debug: serverExecutable,
		transport: TransportKind.stdio,
	}

	switch (context.extensionMode) {
		case ExtensionMode.Development:
		case ExtensionMode.Test:
			return serverOptions
	
		default:
			throw new Error("Production Mode not available at the moment!")
	}
}

function getClientOptions(): LanguageClientOptions {
	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [
			{ scheme: 'file', language: 'fume' },
			{ scheme: 'untitled', language: 'fume' },
		],
		synchronize: {
			// Notify the server about file changes to '.fm files contained in the workspace
			fileEvents: [
				workspace.createFileSystemWatcher('**/.fm'),
				workspace.createFileSystemWatcher("**/*.fm"),
			]
		}
	}

	return clientOptions
}