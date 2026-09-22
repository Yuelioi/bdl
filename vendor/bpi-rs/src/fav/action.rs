use super::info::FavFolderInfo;
use crate::BilibiliRequest;
use crate::fav::FavClient;
use crate::fav::params::{
    FavFolderAddParams, FavFolderDeleteParams, FavFolderEditParams, FavResourceBatchDeleteParams,
    FavResourceCleanParams, FavResourceTransferParams,
};
use crate::response::BpiResult;

const FOLDER_ADD_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/add";
const FOLDER_EDIT_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/edit";
const FOLDER_DEL_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/del";
const RESOURCE_COPY_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/copy";
const RESOURCE_MOVE_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/move";
const RESOURCE_BATCH_DEL_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/batch-del";
const RESOURCE_CLEAN_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/clean";

impl<'a> FavClient<'a> {
    /// 创建收藏夹并返回标准 payload 结果。
    pub async fn add_folder(&self, params: FavFolderAddParams) -> BpiResult<FavFolderInfo> {
        let csrf = self.client.csrf()?;
        self.client
            .post(FOLDER_ADD_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.folder.add")
            .await
    }

    /// 编辑收藏夹并返回标准 payload 结果。
    pub async fn edit_folder(&self, params: FavFolderEditParams) -> BpiResult<FavFolderInfo> {
        let csrf = self.client.csrf()?;
        self.client
            .post(FOLDER_EDIT_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.folder.edit")
            .await
    }

    /// 删除收藏夹并返回标准 payload 结果。
    pub async fn delete_folders(&self, params: FavFolderDeleteParams) -> BpiResult<i32> {
        let csrf = self.client.csrf()?;
        self.client
            .post(FOLDER_DEL_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.folder.delete")
            .await
    }

    /// 复制收藏资源并返回标准 payload 结果。
    pub async fn copy_resources(&self, params: FavResourceTransferParams) -> BpiResult<i32> {
        let csrf = self.client.csrf()?;
        self.client
            .post(RESOURCE_COPY_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.resource.copy")
            .await
    }

    /// 移动收藏资源并返回标准 payload 结果。
    pub async fn move_resources(&self, params: FavResourceTransferParams) -> BpiResult<i32> {
        let csrf = self.client.csrf()?;
        self.client
            .post(RESOURCE_MOVE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.resource.move")
            .await
    }

    /// 删除收藏资源并返回标准 payload 结果。
    pub async fn delete_resources(&self, params: FavResourceBatchDeleteParams) -> BpiResult<i32> {
        let csrf = self.client.csrf()?;
        self.client
            .post(RESOURCE_BATCH_DEL_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.resource.batch_delete")
            .await
    }

    /// 清理失效收藏资源并返回标准 payload 结果。
    pub async fn clean_resources(&self, params: FavResourceCleanParams) -> BpiResult<i32> {
        let csrf = self.client.csrf()?;

        self.client
            .post(RESOURCE_CLEAN_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("fav.resource.clean")
            .await
    }
}

#[cfg(test)]
mod tests {}
