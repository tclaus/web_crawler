use elastic::client::Client;
use elastic::http::sender::SyncSender;
use elastic::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
#[elastic(index = "crawler_index")]
#[derive(ElasticType, Serialize, Deserialize)]
pub struct WebDocument {
    pub title: String,
    pub url: String,
    pub description: String,
    // Tags, Plain Text of body (part of it)
}

fn client() -> Client<SyncSender> {
    let builder = SyncClientBuilder::new()
        .static_node("http://localhost:9200")
        .params_fluent(move |p| p.url_param("pretty", true));

    builder.build().unwrap()
}

fn ensure_indexed(client: &Client<SyncSender>, doc: WebDocument) {
    if !client
        .index(WebDocument::static_index())
        .exists()
        .send()
        .unwrap()
        .exists()
    {
        client
            .index(WebDocument::static_index())
            .create()
            .send()
            .err();

        // Add the document mapping (optional, but makes sure `timestamp` is mapped as a `date`)
        client.document::<WebDocument>().put_mapping().send().err();
    }

    // println!(" Indexing of {:?}...", &doc);
    // Index the document
    let result = client.document().index(doc).send();
    match result {
        Ok(_) => println!(" Index Ok"),
        Err(value) => println!(" Index result: {}", value),
    }
}

/// Add serializable Document to index
pub fn add_to_index(document: WebDocument) {
    let client = client();
    ensure_indexed(&client, document);
}
