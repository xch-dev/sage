use std::mem;

use chia::{
    protocol::{Bytes32, Coin},
    puzzles::offer::{NotarizedPayment, Payment, SettlementPaymentsSolution},
};
use chia_wallet_sdk::{
    driver::{Layer, SettlementLayer, Spend, SpendContext, SpendWithConditions, StandardLayer},
    types::Conditions,
};

use crate::WalletError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum P2Selection {
    Payment,
    Standard,
    Offer(Bytes32),
}

impl P2Selection {
    pub fn nonce(&self) -> Bytes32 {
        match self {
            Self::Payment | Self::Standard => Bytes32::default(),
            Self::Offer(nonce) => *nonce,
        }
    }

    pub fn matches(&self, p2: &P2) -> bool {
        matches!(
            (self, p2),
            (Self::Payment, _) | (Self::Standard, P2::Standard(_)) | (Self::Offer(_), P2::Offer(_))
        )
    }
}

#[derive(Debug, Clone)]
pub enum P2 {
    Standard(StandardP2),
    Offer(SettlementP2),
}

impl P2 {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Standard(p2) => p2.conditions.is_empty(),
            Self::Offer(p2) => p2.notarized_payments.is_empty(),
        }
    }

    pub fn is_standard(&self) -> bool {
        matches!(self, Self::Standard(_))
    }

    pub fn is_offer(&self) -> bool {
        matches!(self, Self::Offer(_))
    }

    pub fn as_standard(&self) -> Option<&StandardP2> {
        match self {
            Self::Standard(p2) => Some(p2),
            Self::Offer(_) => None,
        }
    }

    pub fn as_offer(&self) -> Option<&SettlementP2> {
        match self {
            Self::Standard(_) => None,
            Self::Offer(p2) => Some(p2),
        }
    }

    pub fn as_standard_mut(&mut self) -> Option<&mut StandardP2> {
        match self {
            Self::Standard(p2) => Some(p2),
            Self::Offer(_) => None,
        }
    }

    pub fn as_offer_mut(&mut self) -> Option<&mut SettlementP2> {
        match self {
            Self::Standard(_) => None,
            Self::Offer(p2) => Some(p2),
        }
    }

    #[must_use]
    pub fn cleared(&self) -> Self {
        match self {
            Self::Standard(p2) => Self::Standard(StandardP2::new(p2.layer)),
            Self::Offer(..) => Self::Offer(SettlementP2::new()),
        }
    }

    pub fn inner_spend(&self, ctx: &mut SpendContext) -> Result<Spend, WalletError> {
        match self {
            Self::Standard(p2) => p2.inner_spend(ctx),
            Self::Offer(p2) => p2.inner_spend(ctx),
        }
    }

    pub fn spend(&self, ctx: &mut SpendContext, coin: Coin) -> Result<(), WalletError> {
        match self {
            Self::Standard(p2) => p2.spend(ctx, coin),
            Self::Offer(p2) => p2.spend(ctx, coin),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StandardP2 {
    layer: StandardLayer,
    conditions: Conditions,
}

impl StandardP2 {
    pub fn new(layer: StandardLayer) -> Self {
        Self {
            layer,
            conditions: Conditions::new(),
        }
    }

    pub fn add_conditions(&mut self, conditions: Conditions) {
        self.conditions = mem::take(&mut self.conditions).extend(conditions);
    }

    pub fn inner_spend(&self, ctx: &mut SpendContext) -> Result<Spend, WalletError> {
        Ok(self
            .layer
            .spend_with_conditions(ctx, self.conditions.clone())?)
    }

    pub fn spend(&self, ctx: &mut SpendContext, coin: Coin) -> Result<(), WalletError> {
        self.layer.spend(ctx, coin, self.conditions.clone())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SettlementP2 {
    layer: SettlementLayer,
    notarized_payments: Vec<NotarizedPayment>,
}

impl Default for SettlementP2 {
    fn default() -> Self {
        Self {
            layer: SettlementLayer,
            notarized_payments: Vec::new(),
        }
    }
}

impl SettlementP2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_payment(&mut self, nonce: Bytes32, payment: Payment) {
        if let Some(np) = self
            .notarized_payments
            .iter_mut()
            .find(|np| np.nonce == nonce)
        {
            np.payments.push(payment);
        } else {
            self.notarized_payments.push(NotarizedPayment {
                nonce,
                payments: vec![payment],
            });
        }
    }

    pub fn inner_spend(&self, ctx: &mut SpendContext) -> Result<Spend, WalletError> {
        Ok(self.layer.construct_spend(
            ctx,
            SettlementPaymentsSolution {
                notarized_payments: self.notarized_payments.clone(),
            },
        )?)
    }

    pub fn spend(&self, ctx: &mut SpendContext, coin: Coin) -> Result<(), WalletError> {
        let coin_spend = self.layer.construct_coin_spend(
            ctx,
            coin,
            SettlementPaymentsSolution {
                notarized_payments: self.notarized_payments.clone(),
            },
        )?;
        ctx.insert(coin_spend);
        Ok(())
    }
}
