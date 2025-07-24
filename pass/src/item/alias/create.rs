use crate::PassClient;
use crate::constants::ITEM_CONTENT_CONTENT_FORMAT_VERSION;
use crate::item::list::ItemRevision;
use anyhow::{Context, Result, anyhow};
use muon::POST;
use pass_domain::{AliasItem, ItemContent, ItemData, ItemId, ShareId, crypto};

#[derive(Debug)]
pub struct CreatedAliasItem {
    pub alias: String,
    pub item_id: ItemId,
}

#[derive(serde::Serialize)]
struct CreateItemRequest {
    #[serde(rename = "KeyRotation")]
    pub key_rotation: u8,
    #[serde(rename = "ContentFormatVersion")]
    pub content_format_version: u32,
    #[serde(rename = "Content")]
    pub content: String,
    #[serde(rename = "ItemKey")]
    pub item_key: String,
}

#[derive(serde::Serialize)]
struct CreateAliasRequest {
    #[serde(rename = "Prefix")]
    pub prefix: String,
    #[serde(rename = "SignedSuffix")]
    pub signed_suffix: String,
    #[serde(rename = "MailboxIDs")]
    pub mailbox_ids: Vec<i64>,
    #[serde(rename = "AliasName")]
    pub alias_name: Option<String>,
    #[serde(rename = "Item")]
    pub item: CreateItemRequest,
}

#[derive(serde::Deserialize)]
struct CreateItemResponse {
    #[serde(rename = "Item")]
    pub item: ItemRevision,
}

impl PassClient {
    pub async fn create_alias(&self, share_id: &ShareId, prefix: &str) -> Result<CreatedAliasItem> {
        let request = self
            .create_alias_request(share_id, prefix)
            .await
            .context("Error creating create_alias request")?;

        let req = POST!("/pass/v1/share/{share_id}/alias/custom")
            .body_json(request)
            .context("Error serializing create_alias request")?;
        let res = self
            .client
            .send(req)
            .await
            .context("Error sending create alias request")?;
        let response: CreateItemResponse = assert_response!(res);

        let email = match response.item.alias_email {
            Some(email) => email,
            None => return Err(anyhow!("Error getting email from created alias")),
        };
        Ok(CreatedAliasItem {
            alias: email,
            item_id: ItemId::new(response.item.item_id),
        })
    }

    async fn create_alias_request(
        &self,
        share_id: &ShareId,
        prefix: &str,
    ) -> Result<CreateAliasRequest> {
        let mut options = self
            .get_alias_options(share_id)
            .await
            .context("Error fetching alias options")?;

        let suffix = options.suffixes.pop().context("No suffix found")?;
        let mailbox = options.mailboxes.pop().context("No mailbox found")?;

        let title = format!("Alias for {prefix}");
        let item = self
            .create_alias_item_request(share_id, &title)
            .await
            .context("Error creating create_alias_item request")?;

        Ok(CreateAliasRequest {
            prefix: prefix.to_string(),
            signed_suffix: suffix.signed_suffix,
            mailbox_ids: vec![mailbox.id],
            alias_name: None,
            item,
        })
    }

    async fn create_alias_item_request(
        &self,
        share_id: &ShareId,
        title: &str,
    ) -> Result<CreateItemRequest> {
        let share_keys = self
            .get_share_keys(share_id)
            .await
            .context("Error retrieving share keys")?;

        let share_key = share_keys.latest_or_err()?;

        let item_key = crypto::generate_encryption_key();

        let content = ItemData {
            title: title.to_string(),
            note: "".to_string(),
            item_uuid: ItemData::generate_uuid(),
            content: ItemContent::Alias(AliasItem),
            extra_fields: vec![],
        };
        let serialized_content = content
            .serialize()
            .context("Error serializing item content")?;
        let encrypted_item_content = crypto::encrypt(
            &serialized_content,
            &item_key,
            crypto::EncryptionTag::ItemContent,
        )
        .map_err(|e| {
            error!("Error encrypting item contents: {e}");
            anyhow!("Error encrypting item contents")
        })?;

        let opened_share_key = self
            .open_share_key(share_key.clone())
            .await
            .context("Error opening share key")?;

        let encrypted_item_key = crypto::encrypt(
            &item_key,
            opened_share_key.as_ref(),
            crypto::EncryptionTag::ItemKey,
        )
        .map_err(|e| {
            error!("Error encrypting item key: {e}");
            anyhow!("Error encrypting item key")
        })?;

        Ok(CreateItemRequest {
            key_rotation: share_key.key_rotation,
            content_format_version: ITEM_CONTENT_CONTENT_FORMAT_VERSION,
            content: crate::utils::b64_encode(encrypted_item_content),
            item_key: crate::utils::b64_encode(encrypted_item_key),
        })
    }
}
