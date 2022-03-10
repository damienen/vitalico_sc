#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::contract]
pub trait EgldEsdtSwap {
    #[init]
    fn init(&self) {
        self.is_paused().set(true)
    }

    // endpoints - owner-only

    #[only_owner]
    #[payable("*")]
    #[endpoint(fundWithToken)]
    fn fund_with_token(&self,token_price:BigUint){
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(self.token_identifier().is_empty()||self.token_identifier().get()==payment_token,"contract already has another token");
        require!(payment_token != TokenIdentifier::egld(), "Can't pay with eGLD");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        self.token_price().set(token_price);
        self.token_identifier().set(&payment_token);
    }

    #[only_owner]
    #[endpoint(defundContractTokens)]
    fn defund_contract_tokens(&self){
        let token_identifier = self.token_identifier().get();
        require!(
            self.get_token_balance()>BigUint::zero(),
            "contract has no tokens"
        );
        require!(
            !token_identifier.is_empty(),
            "contract has no token id"
        );
        let balance=self.get_token_balance();
        let caller = self.blockchain().get_caller();
        self.send().direct(&caller, &token_identifier, 0, &balance, &[]);
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(fundWithEgld)]
    fn fund_with_egld(&self){
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        require!(payment_token == TokenIdentifier::egld(), "Can only pay with eGLD");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
    }

    #[only_owner]
    #[endpoint(defundContractEgld)]
    fn defund_contract_egld(&self){
        require!(
            self.get_egld_balance()!=0u32,
            "contract has no egld"
        );
        let balance=self.get_egld_balance();
        let caller = self.blockchain().get_caller();
        self.send().direct(&caller, &TokenIdentifier::egld(), 0, &balance, &[]);
    }

    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self){
        let pause_value=&self.is_paused().get();
        if self.is_paused().is_empty(){
            self.is_paused().set(true);
        }else{
            self.is_paused().set(!pause_value);
        }
        
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(buyToken)]
    fn buy_token(&self){
        require!(self.is_paused().get()==false,"Contract is paused");
        let caller = self.blockchain().get_caller();
        let token_identifier = self.token_identifier().get();
        let (payment_amount, payment_token) = self.call_value().payment_token_pair();
        let denominator=BigUint::from(10u32).pow(8);
        let amount_to_send=&payment_amount/&self.token_price().get()*&denominator;
        
        require!(payment_token.is_egld(), "Only EGLD accepted");
        require!(payment_amount > 0u32, "Payment must be more than 0");
        require!(
            self.get_token_balance()>=&payment_amount/&self.token_price().get()*&denominator,
            "contract doesn't have enough tokens"
        );
        let den=BigUint::from(1000000000u32)*&denominator;
        require!(
            &amount_to_send%&den==0u32,
            "can only buy multiple of 1.000.000.000"
        );
        self.send().direct(&caller, &token_identifier, 0, &amount_to_send, &[]);
        self.send().direct(&self.blockchain().get_owner_address(),&payment_token,0,&payment_amount,&[])
    }

    #[view(getEgldBalance)]
    fn get_egld_balance(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&TokenIdentifier::egld(), 0)
    }
    #[view(getTokenBalance)]
    fn get_token_balance(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&self.token_identifier().get(), 0)
    }

    // storage

    #[view(getTokenIdentifier)]
    #[storage_mapper("TokenId")]
    fn token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getTokenPrice)]
    #[storage_mapper("TokenPrice")]
    fn token_price(&self) -> SingleValueMapper<BigUint>;

    #[view(isPaused)]
    #[storage_mapper("isPaused")]
    fn is_paused(&self) -> SingleValueMapper<bool>;
    // events
}
