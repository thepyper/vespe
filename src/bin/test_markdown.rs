use markdown::{to_mdast, ParseOptions};
use markdown::message::Message;

fn main()  -> Result<(), Message>  {
     
    let examples = vec![
r###"# Hi *Earth*!"###,
r###"
# Hi *Earth*!!!
This is line 2

This is line 4

This is line 6
"###,        
r###"
# Objective <!-- my note -->

# HeyHey

haha haha haha

"###,
r###"
some text
some more text

some more more text

# section

bababababa

"###
    ];
    
    for (i, example) in examples.into_iter().enumerate() {
        let tree = to_mdast(example, &ParseOptions::default())?;
        println!("example {i}:\n{:?}", tree);        
        println!("string  {i}:\n{}", tree.to_string());
    }
    
    Ok(())
}