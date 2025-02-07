use crate::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(
        schemas(
            // Keys
            Login, LoginResponse,
            Logout, LogoutResponse,
            Resync, ResyncResponse,
            GenerateMnemonic, GenerateMnemonicResponse,
            ImportKey, ImportKeyResponse,
            DeleteKey, DeleteKeyResponse,
            RenameKey, RenameKeyResponse,
            GetKey, GetKeyResponse,
            GetSecretKey, GetSecretKeyResponse,
            GetKeys, GetKeysResponse,
            // Offers
            MakeOffer, MakeOfferResponse,
            TakeOffer, TakeOfferResponse,
            CombineOffers, CombineOffersResponse,
            ViewOffer, ViewOfferResponse,
            ImportOffer, ImportOfferResponse,
            GetOffers, GetOffersResponse,
            GetOffer, GetOfferResponse,
            DeleteOffer, DeleteOfferResponse,
            CancelOffer, CancelOfferResponse,
            // Add other types as needed...
        )
    ),
    tags(
        (name = "keys", description = "Key management endpoints"),
        (name = "offers", description = "Offer management endpoints"),
        // Add other tags as needed...
    )
)]
#[derive(Copy, Clone, Debug)]
pub struct ApiDoc;
