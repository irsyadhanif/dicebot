#[macro_use]
extern crate serenity;
extern crate typemap;
extern crate rand;
extern crate core;

use serenity::client::Context;
use serenity::Client;
use serenity::model::{Message, Game};
use serenity::ext::framework::help_commands;
use serenity::utils::MessageBuilder;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::num::ParseIntError;
use core::str;
use typemap::Key;

struct GameName;
struct DM;

impl Key for GameName {
    type Value = String;
}

impl Key for DM {
    type Value = serenity::model::User;
}
//---------------------------
//---------GAMES-------------
//---------------------------
fn shadowrun(times: i64, msg: &Message) {
    let rolls = roll_dice(6, times);
    if let Err(why) = msg.channel_id.say(&format!("Rolling for Shadowrun: {:?}", rolls)) {
        println!("Error sending message {:?}", why);
    }
    let mut ones = 0;
    let mut hits = 0;
    for result in rolls {
        match result {
            1     => ones += 1,
            5 | 6 => hits += 1,
            _     => (),
        }
    }
    //msg.channel_id.say(&format!("Ones: {}", ones));
    //msg.channel_id.say(&format!("Hits: {}", hits));
    let response = MessageBuilder::new()
        .push(&format!("Ones: {}\n", ones))
        .push(&format!("Hits: {}", hits));
    if let Err(why) = msg.channel_id.say(&response.build()) {
        println!("Error sending message {:?}", why);
    }
}

fn wod(times: i64, msg: &Message) {
    let rolls = roll_dice(10, times);
    if let Err(why) = msg.channel_id.say(&format!("Rolling for World of Darkness: {:?}", rolls)) {
        println!("Error sending message {:?}", why);
    }
    let mut ones = 0;
    let mut hits = 0;
    let mut tens = 0;
    for result in rolls {
        match result {
            1     => ones += 1,
            8 | 9 => hits += 1,
            10    => { hits +=1; tens += 1 }
            _     => (),
        }
    }
    if tens > 0 {
        if let Err(why) = msg.channel_id.say(&format!("Roll again!")) {
            println!("Error sending message {:?}", why);
        }
        wod(tens, & msg);
    }
    //msg.channel_id.say(&format!("Ones: {}", ones));
    //msg.channel_id.say(&format!("Hits: {}", hits));
    let response = MessageBuilder::new()
        .push(&format!("Ones: {}\n", ones))
        .push(&format!("Hits: {}", hits));
    if let Err(why) = msg.channel_id.say(&response.build()) {
        println!("Error sending message {:?}", why);
    }
}

fn ore(times: i64, msg: &Message) {
    let rolls = roll_dice(10, times);
    let mut results = vec![0; 10];
    let mut pairs = Vec::new();
    if let Err(why) = msg.channel_id.say(&format!("Rolling for One-Roll Engine: {:?}", rolls)) {
        println!("Error sending message {:?}", why);
    }
    for roll in rolls {
        let itr = roll as usize;
        results[itr - 1] += 1;
    }
    let mut y = 0;
    for result in results {
        y += 1;
        if result == 0 { continue; }
        if result == 1 { continue; }
        pairs.push(format!("{}x{}", result, y));
    }

    if let Err(why) = msg.channel_id.say(&format!("Sets: {:?}", pairs)) {
        println!("Error sending message {:?}", why);
    }
}
//----------------------------
//----------MAIN--------------
//----------------------------
fn main() {
    // read private key from key.txt
    let path = Path::new("key.txt");

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

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<GameName>(String::from("default"));
        data.insert::<DM>(serenity::model::User{
            bot: false,
            id: serenity::model::UserId::from(0),
            discriminator: String::from(""),
            name: String::from(""),
            avatar: None::<String>,

        });
    }
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
        .command("about", |c| c.exec_str("A dice rolling bot"))
        .command("help", |c| c.exec_help(help_commands::plain))
        .command("ping", |c| c
            .check(owner_check)
            .exec_str("Pong!"))
        .command("roll", |c| c
            .exec(roll)
            .known_as("r")
            .desc("Roll an n-sided dice x times.  Usage: !roll <x>d<n>"))
        .command("config", |c| c
            .desc("Set game for dice rolls.")
            .exec(config))
        .command("playing", |c| c
            .exec(playing)
            .desc("Print current game"))
        .command("rollgame", |c| c
            .desc("Roll dice for set game (use !config to set game)")
            .exec(rollgame)
            .batch_known_as(vec!["rg", "rollg", "rgame"]))
        .command("dmroll", |c| c
            .exec(dmroll)
            .known_as("dmr")
            .desc("Private roll that only you and the DM see.  Requires a set DM."))
        .command("givemedm", |c| c
            .exec(setdm)
            .desc("Sets the DM as yourself.  Allows use of !dmroll"))
        .command("whoisdm", |c| c
            .exec(whoisdm)
            .desc("Check who the DM is.")));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}





//----------------------------
//-------MISC FUNCTIONS-------
//----------------------------

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.

fn owner_check(_: &mut Context, msg: &Message) -> bool {
    // Replace 7 with your ID
    msg.author.id == 117810256209248264
}

fn roll_dice(d: i64, times: i64) -> Vec<i64> {
    let mut rolls = Vec::new();

    for x in 0..times {
        let result = rand::thread_rng().gen_range(1, d + 1);
        rolls.push(result);
    }

    return rolls;
}

fn sti(number_str: &str) -> Result<i64, ParseIntError> {
    match number_str.parse::<i64>() {
        Ok(n) => Ok(n),
        Err(err) => Err(err),
    }
}

fn baka(msg: &Message) {
    if let Err(why2) = msg.channel_id.say(&format!("You baka-ass motherfucker")) {
        println!("Error sending message: {:?}", why2);
    };
}
//----------------------------
//---------COMMANDS-----------
//----------------------------

// rolls a <x> sided die <y> times.
command!(roll(_ctx, msg, args, first: String) {
    let split = first.split("d");
    let numbers = split.collect::<Vec<&str>>();
    let number1 = match sti(numbers[1]) {
        Ok(f) => f,
        Err(why) => { baka(& msg); 0 }

    };
    let number2 = match sti(numbers[0]) {
        Ok(f) => f,
        Err(why) => { baka(& msg); 0 }

    };
    let rolls = roll_dice(number1, number2);
    if let Err(why) = msg.channel_id.say(&format!("Rolls: {:?}", rolls)) {
        println!("Error sending message: {:?}", why);
    }
});

command!(dmroll(_ctx, msg, args) {
    let mut data = _ctx.data.lock().unwrap();
    let dm_user = data.get_mut::<DM>().unwrap();
    if dm_user.name == "" {
        if let Err(why) = msg.channel_id.say(&format!("No Dungeon Master set.")) {
            println!("Error sending message {:?}", why);
        }
    } else {
        let split = args[0].split("d");
        let numbers = split.collect::<Vec<&str>>();
        let number1 = match sti(numbers[1]) {
            Ok(f) => f,
            Err(why) => { baka(& msg); 0 }

        };
        let number2 = match sti(numbers[0]) {
            Ok(f) => f,
            Err(why) => { baka(& msg); 0 }

        };
        let rolls = roll_dice(number1, number2);
        if let Err(why) = msg.channel_id.say(&format!("Rolls: {:?}.  Also sent to DM.", rolls)) {
            println!("Error sending message: {:?}", why);
        }

        if let Err(why) = dm_user.dm(&format!("Roll sent from {:?}", msg.author.name)) {
            println!("Error sending DM: {:?}", why);
        }
        if let Err(why) = dm_user.dm(&format!("Rolls: {:?}", rolls)) {
            println!("Error sending DM: {:?}", why);
        }
    }
});


// sets game name.  will be used to configure dice rules.
command!(config(_ctx, msg, args) {
    let game_name = args.join(" ");
    let game_name2 = args.join(" ");
    _ctx.set_game(Game::playing(& game_name));
    let mut data = _ctx.data.lock().unwrap();
    data.insert::<GameName>(game_name);
    if let Err(why) = msg.channel_id.say(&format!("Configured for {:?}", game_name2)) {
        println!("Error sending message: {:?}", why);
    }
});

// debug command, prints out current game playing
command!(playing(_ctx, msg, args) {
    let mut data = _ctx.data.lock().unwrap();
    let name = data.get_mut::<GameName>().unwrap();
    if let Err(why) = msg.channel_id.say(&format!("I am playing {}", name)) {
        println!("Error sending message {:?}", why);
    }
});

// rolls based on game rules.
command!(rollgame(_ctx, msg, args, first: i64) {
    let mut data = _ctx.data.lock().unwrap();
    let name = data.get_mut::<GameName>().unwrap();
    if name == "shadowrun" {
        shadowrun(first, msg);
    } else if name == "wod" {
        wod(first, msg);
    } else if name == "ore" {
        ore(first, msg);
    } else {
        if let Err(why) = msg.channel_id.say(&format!("No game configured or invalid name")) {
            println!("Error sending message: {:?}", why);
        }
    }

});


command!(setdm(_ctx, msg, args) {
    let dm_user = match serenity::model::User::get(msg.author.id) {
        Ok(t) => t,
        Err(why) => panic!("Something happened: {:?}", why),
    };
    let mut data = _ctx.data.lock().unwrap();
    data.insert::<DM>(dm_user);
    if let Err(why) = msg.channel_id.say(&format!("Dungeon Master set.")) {
        println!("Error sending message {:?}", why);
    }
});

command!(whoisdm(_ctx, msg, args) {
    let mut data = _ctx.data.lock().unwrap();
    let name = data.get_mut::<DM>().unwrap();
    if let Err(why) = msg.channel_id.say(&format!("The Dungeon Master is {:?}.", name.name)) {
        println!("Error sending message {:?}", why);
    }
});
