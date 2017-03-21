#[macro_use]
extern crate serenity;
extern crate typemap;
extern crate rand;

use serenity::client::Context;
use serenity::Client;
use serenity::model::{Message, Game};
use serenity::ext::framework::help_commands;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    // read private key from key.txt
    let path = Path::new("src/key.txt");

    let mut file = match File::open(&path) {
        Err(e) => panic!("Couldn't open key file: {}", e),
        Ok(file) => file,
    };

    let mut key = String::new();
    match file.read_to_string(&mut key){
        Err(e) => panic!("Couldn't read key file: {}", e),
        Ok(key) => key,
    };
    key.pop(); // remove stray newline
    println!("Key is {}", key);

    let mut client = Client::login_bot(& key);


    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });


    client.with_framework(|f| f
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .rate_limit_message("Try this again in `%time%` seconds.")
            .prefix("!"))
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, `set_check` is provided to give you this
        // functionality.
        .before(|ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })
        .command("about", |c| c.exec_str("A test bot"))
        .command("help", |c| c.exec_help(help_commands::plain))
        .command("multiply", |c| c
            .known_as("*") // Lets us call ~* instead of ~multiply
            .exec(multiply))
        .command("ping", |c| c
            .check(owner_check)
            .exec_str("Pong!"))
        .command("roll", |c| c
            .exec(roll)
            .known_as("r")
            .desc("Roll a dx dice y times.  Usage: !roll x y"))
        .command("config", |c| c
            .desc("Set game for dice rolls.")
            .exec(config))
        .command("playing", |c| c
            .exec(playing)
            .desc("Print current game")));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}



// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
fn owner_check(_: &mut Context, msg: &Message) -> bool {
    // Replace 7 with your ID
    msg.author.id == 117810256209248264
}

command!(multiply(_ctx, msg, args, first: f64, second: f64) {
    let res = first * second;

    if let Err(why) = msg.channel_id.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});

// rolls a <x> sided die <y> times.
command!(roll(_ctx, msg, args, first: i64, second: i64) {
    let mut rolls = Vec::new();

    for x in 0..second {
        let result = rand::thread_rng().gen_range(1, first + 1);
        rolls.push(result);
    }


    if let Err(why) = msg.channel_id.say(&format!("Rolls: {:?}", rolls)) {
        println!("Error sending message: {:?}", why);
    }
});

// sets game name.  will be used to configure dice rules.
command!(config(_ctx, msg, args) {
    let game_name = args.join(" ");
    _ctx.set_game(Game::playing(& game_name));
    if let Err(why) = msg.channel_id.say(&format!("Configured for {:?}", game_name)) {
        println!("Error sending message: {:?}", why);
    }
});

// debug command, prints out current game playing (non-functional)
command!(playing(_ctx, msg, args) {
    let mut name = "";
    if let Err(why) = msg.channel_id.say(&format!("I am playing {}", name)) {
        println!("Error sending message {:?}", why);
    }
});
