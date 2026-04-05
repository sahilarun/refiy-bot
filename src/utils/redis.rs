use redis::AsyncCommands;
use crate::data::GuildPlayer;
use crate::error::BotError;

pub async fn savePlayer(
    con: &mut redis::aio::ConnectionManager,
    guild_id: u64,
    player: &GuildPlayer,
) -> Result<(), BotError> {
    let key = format!("player:{}", guild_id);
    let json = serde_json::to_string(player).map_err(|e| BotError::Other(e.to_string()))?;
    
    con.set::<_, _, ()>(key, json).await.map_err(|e| BotError::Other(e.to_string()))?;
    Ok(())
}

pub async fn loadPlayer(
    con: &mut redis::aio::ConnectionManager,
    guild_id: u64,
) -> Result<Option<GuildPlayer>, BotError> {
    let key = format!("player:{}", guild_id);
    let json: Option<String> = con.get(key).await.map_err(|e| BotError::Other(e.to_string()))?;
    
    match json {
        Some(s) => Ok(Some(serde_json::from_str(&s).map_err(|e| BotError::Other(e.to_string()))?)),
        None => Ok(None),
    }
}

pub async fn deletePlayer(
    con: &mut redis::aio::ConnectionManager,
    guild_id: u64,
) -> Result<(), BotError> {
    let key = format!("player:{}", guild_id);
    con.del::<_, ()>(key).await.map_err(|e| BotError::Other(e.to_string()))?;
    Ok(())
}

pub async fn loadAllPlayers(
    con: &mut redis::aio::ConnectionManager,
) -> Result<Vec<(u64, GuildPlayer)>, BotError> {
    let keys: Vec<String> = redis::cmd("KEYS").arg("player:*").query_async(con).await.map_err(|e| BotError::Other(e.to_string()))?;
    
    let mut players = Vec::new();
    for key in keys {
        if let Some(id_str) = key.strip_prefix("player:") {
            if let Ok(guild_id) = id_str.parse::<u64>() {
                let json: Option<String> = con.get(&key).await.map_err(|e| BotError::Other(e.to_string()))?;
                if let Some(s) = json {
                    if let Ok(player) = serde_json::from_str::<GuildPlayer>(&s) {
                        players.push((guild_id, player));
                    }
                }
            }
        }
    }
    
    Ok(players)
}
