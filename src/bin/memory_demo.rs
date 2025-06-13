use project_x::orchestrator::memory::{MemoryManager, MessageRole};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🧠 Memory Manager Demo");
    
    // Create a new memory manager
    let mut memory = MemoryManager::new(Some("demo_memory.sqlite"))?;
    
    // Add some sample conversation
    println!("\n📝 Adding sample conversation...");
    memory.add_message(MessageRole::User, "How do I create a Rust function?")?;
    memory.add_message(MessageRole::Assistant, "You can create a Rust function using the 'fn' keyword. Here's an example:\n\nfn hello_world() {\n    println!(\"Hello, world!\");\n}")?;
    memory.add_message(MessageRole::User, "What about function parameters?")?;
    memory.add_message(MessageRole::Assistant, "Function parameters are defined in parentheses after the function name. For example:\n\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}")?;
    memory.add_message(MessageRole::User, "Can you show me how to use structs?")?;
    
    // Get recent history
    println!("\n📚 Recent conversation history (last 3 messages):");
    let recent = memory.get_recent_history(3)?;
    for message in recent {
        println!("{}: {}", message.role, 
                if message.content.len() > 80 {
                    format!("{}...", &message.content[..80])
                } else {
                    message.content
                });
    }
    
    // Search messages
    println!("\n🔍 Searching for messages containing 'function':");
    let search_results = memory.search_messages("function", 5)?;
    for message in search_results {
        println!("{}: {}", message.role, 
                if message.content.len() > 60 {
                    format!("{}...", &message.content[..60])
                } else {
                    message.content
                });
    }
    
    // Get statistics
    println!("\n📊 Memory Statistics:");
    let stats = memory.get_stats()?;
    println!("Total messages: {}", stats.total_messages);
    println!("User messages: {}", stats.user_messages);
    println!("Assistant messages: {}", stats.assistant_messages);
    if let Some(first) = stats.first_message {
        println!("First message: {}", first.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    if let Some(last) = stats.last_message {
        println!("Last message: {}", last.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    // Format context for LLM
    println!("\n🤖 Formatted context for LLM:");
    let context = memory.format_context(3)?;
    println!("{}", context);
    
    println!("\n✅ Memory demo complete! Database saved to demo_memory.sqlite");
    
    Ok(())
}