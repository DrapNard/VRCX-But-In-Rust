use crate::{
    api::PaginationQuery,
    client::VrcClient,
    error::VrcError,
    models::{
        economy::{
            ActiveLicense, Balance, BalanceEarnings, EconomyAccount, LicenseGroup, ProductListing,
            ProductListingList, ProductPurchase, ProductPurchaseCreate, Store, StoreShelf,
            SubscriptionList, TiliaStatus, TiliaTos, TiliaTosUpdate, TokenBundle,
        },
        users::UserEligibility,
    },
};

impl VrcClient {
    pub async fn active_licenses(&self) -> Result<Vec<ActiveLicense>, VrcError> {
        self.get_json("economy/licenses/active").await
    }

    pub async fn license_group(&self, id: &str) -> Result<LicenseGroup, VrcError> {
        self.get_json(&format!("licenseGroups/{id}")).await
    }

    pub async fn store(&self) -> Result<Store, VrcError> {
        self.get_json("economy/store").await
    }

    pub async fn stores(&self, query: &PaginationQuery) -> Result<Vec<Store>, VrcError> {
        self.get_json_with_query("economy/stores", query).await
    }

    pub async fn store_shelves(&self) -> Result<Vec<StoreShelf>, VrcError> {
        self.get_json("economy/store/shelves").await
    }

    pub async fn product_listings(
        &self,
        query: &PaginationQuery,
    ) -> Result<ProductListingList, VrcError> {
        self.get_json_with_query("listing", query).await
    }

    pub async fn product_listing(&self, product_id: &str) -> Result<ProductListing, VrcError> {
        self.get_json(&format!("listing/{product_id}")).await
    }

    pub async fn purchase_listing(
        &self,
        body: &ProductPurchaseCreate,
    ) -> Result<ProductPurchase, VrcError> {
        self.post_json("economy/purchase/listing", body).await
    }

    pub async fn purchases(
        &self,
        query: &PaginationQuery,
    ) -> Result<Vec<ProductPurchase>, VrcError> {
        self.get_json_with_query("economy/purchases", query).await
    }

    pub async fn purchase(&self, purchase_id: &str) -> Result<ProductPurchase, VrcError> {
        self.get_json(&format!("economy/purchases/{purchase_id}"))
            .await
    }

    pub async fn subscriptions(&self) -> Result<SubscriptionList, VrcError> {
        self.get_json("subscriptions").await
    }

    pub async fn current_subscription(&self) -> Result<SubscriptionList, VrcError> {
        self.get_json("auth/user/subscription").await
    }

    pub async fn token_bundles(&self) -> Result<Vec<TokenBundle>, VrcError> {
        self.get_json("tokenBundles").await
    }

    pub async fn user_balance(&self, user_id: &str) -> Result<Balance, VrcError> {
        self.get_json(&format!("user/{user_id}/balance")).await
    }

    pub async fn user_earnings(&self, user_id: &str) -> Result<BalanceEarnings, VrcError> {
        self.get_json(&format!("user/{user_id}/balance/earnings"))
            .await
    }

    pub async fn economy_account(&self, user_id: &str) -> Result<EconomyAccount, VrcError> {
        self.get_json(&format!("user/{user_id}/economy/account"))
            .await
    }

    pub async fn tilia_status(&self) -> Result<TiliaStatus, VrcError> {
        self.get_json("tilia/status").await
    }

    pub async fn tilia_tos(&self, user_id: &str) -> Result<TiliaTos, VrcError> {
        self.get_json(&format!("user/{user_id}/tilia/tos")).await
    }

    pub async fn update_tilia_tos(
        &self,
        user_id: &str,
        body: &TiliaTosUpdate,
    ) -> Result<TiliaTos, VrcError> {
        self.put_json(&format!("user/{user_id}/tilia/tos"), body)
            .await
    }

    pub async fn credit_eligibility(&self, user_id: &str) -> Result<UserEligibility, VrcError> {
        self.get_json(&format!("users/{user_id}/credits/eligible"))
            .await
    }
}
