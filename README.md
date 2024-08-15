# ScammEd: the not so real editor.

This is useful when making videos and you don't want to type terminal commands or write code.

To get started, first create a new scene. In this scene, every new line represents a new action. There are 4 types of actions: Fake Edit File, Run Command Quiet, Run Command without Commandline and Run Command.

If you, for example, want to create a new directory and cd into there without display it, you can add a quiet mkdir and cd command. Quiet commands are just very simply `#command`. This means that the command nor the output will be printed to the console.

For the sake of tutorial, you might also want to print text to the screen. You can use the Run command without commandline action for this. Similar to a quiet command, you can designate a command as such by prefixing it with `-`, for example `-echo Hello, World!`. This means only the output will be printed to the screen.

Now, you need to actually show the users which command you're running. An example for this is initialising a project, for example using `cargo init . --name "my name"`. If you don't prefix your command with anything, this will make it actually be printed to the screen alongside its output (Format: `<green>path $ <blue>command_name<white>args...`).

Finally, you also want to add some code. For this, simply run the built-in fake edit file action. This you can do by using the `+`-designator. Simply type `+` and pass as the 2 arguments the file you want to edit (iex. `./src/main.rs`) and the file you want to pull the code from (iex. `../src/main.rs`). This will copy the code over, overwriting any existing files and launch the ScammEd editor. This editor will start writing about 1 second after being opened. It will write until it hits a `//[WAIT]` in your code. After that, it will stop and wait for a keypress to continue. Once it reaches the end of the file, it waits for another keypress before exiting and executing the next action as defined in the scene. See below for some example code with breaks.


--- OLD ---

This is useful when making videos and you want to display some code on screen
but you don't want to type it out (because editing typos is annoying).

Add `//[WAIT]` in your code to pause the printer.

E.g

```rust
//[WAIT]
fn main() {
//[WAIT]
    println!("hello, world");
//[WAIT]
    println!("hello, again");
}
```
