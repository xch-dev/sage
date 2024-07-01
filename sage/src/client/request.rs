use chia::protocol::{
    PuzzleSolutionResponse, RejectCoinState, RejectPuzzleSolution, RejectPuzzleState,
    RequestCoinState, RequestPuzzleSolution, RequestPuzzleState, RequestRemoveCoinSubscriptions,
    RequestRemovePuzzleSubscriptions, RespondCoinState, RespondPuzzleSolution, RespondPuzzleState,
    RespondRemovePuzzleSubscriptions,
};
use std::future::Future;

use crate::{Peer, Response, Result};

pub trait Request<Body> {
    type Response;

    fn request(&self, body: Body) -> impl Future<Output = Result<Self::Response>> + Send;
}

impl Request<RequestPuzzleSolution> for Peer {
    type Response = Response<PuzzleSolutionResponse, RejectPuzzleSolution>;

    async fn request(&self, body: RequestPuzzleSolution) -> Result<Self::Response> {
        let response: Response<RespondPuzzleSolution, RejectPuzzleSolution> =
            self.request_fallible(body).await?;
        Ok(match response {
            Response::Success(response) => Response::Success(response.response),
            Response::Rejection(rejection) => Response::Rejection(rejection),
        })
    }
}

impl Request<RequestRemovePuzzleSubscriptions> for Peer {
    type Response = RespondRemovePuzzleSubscriptions;

    async fn request(&self, body: RequestRemovePuzzleSubscriptions) -> Result<Self::Response> {
        self.request_infallible(body).await
    }
}

impl Request<RequestRemoveCoinSubscriptions> for Peer {
    type Response = RequestRemoveCoinSubscriptions;

    async fn request(&self, body: RequestRemoveCoinSubscriptions) -> Result<Self::Response> {
        self.request_infallible(body).await
    }
}

impl Request<RequestPuzzleState> for Peer {
    type Response = Response<RespondPuzzleState, RejectPuzzleState>;

    async fn request(&self, body: RequestPuzzleState) -> Result<Self::Response> {
        self.request_fallible(body).await
    }
}

impl Request<RequestCoinState> for Peer {
    type Response = Response<RespondCoinState, RejectCoinState>;

    async fn request(&self, body: RequestCoinState) -> Result<Self::Response> {
        self.request_fallible(body).await
    }
}
