use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::Mentionable;
use serenity::utils::MessageBuilder;
use std::env;

mod futuremorse;
mod WavSink;
use futuresdr::async_io::block_on;


#[group]
#[commands(ping, cw)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    // let application_id: u64 = env::var("APPLICATION_ID")
    //     .expect("Expected an application id in the environment")
    //     .parse()
    //     .expect("application id is not a valid id");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("||/")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        //.application_id(application_d)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    // let channel = match msg.channel_id.to_channel(&context).await {
    //     Ok(channel) => channel,
    //     Err(why) => {
    //         println!("Error getting channel: {:?}", why);

    //         return;
    //     },
    // };

    let response = MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&msg.author.name)
        .push(" used the 'ping' command in the ")
        //.mention(&channel)
        //.push(" channel")
        .build();

    msg.reply(ctx, response).await?;

    Ok(())
}

#[command]
async fn cw(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text_input = format!("{}", args.rest());
    let mut msg_to_encode = text_input.as_str();
    if text_input.ends_with("||") {
        msg_to_encode = &msg_to_encode[..msg_to_encode.len()-2];
    }
    let morse_msg = &morse::encode::encode(msg_to_encode).unwrap();

    let base_filename = base64::encode(msg_to_encode);
    let wav_filename = format!("{}.wav", base_filename);
    block_on(futuremorse::run_fg(msg_to_encode, &wav_filename));
    let mp3_filename = format!("{}.mp3", base_filename);
    
    let status = std::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(&*wav_filename)
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-qscale:a")
        .arg("2")
        .arg(&*mp3_filename)
        .status()
        .expect("Cannot convert to mp3");

        println!("process finished with: {}", status);

assert!(status.success());

    let title = format!("User {} is emitting", msg.author.id.mention());
    msg.channel_id
        .send_message(ctx, |m| {
            m.content("")
                .embed(|e| {
                    e.title("CW").description(title).fields(vec![
                        ("morse",  &format!("``` {} ```", morse_msg), false),
                        ("texte", &format!("|| {} ||", msg_to_encode), false),
                    ]).attachment(&format!("attachment://{}",mp3_filename))
                })
                .reference_message(msg)
                .add_file(&*mp3_filename)
        })
        .await?;


    let status = std::process::Command::new("rm")
        .arg("-f")
        .arg(&*wav_filename)
        .arg(&*mp3_filename)
        .status()
        .expect("Cannot delete files");

    Ok(())
}
