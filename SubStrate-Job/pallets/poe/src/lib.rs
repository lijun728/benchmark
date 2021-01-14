#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::Get,ensure, StorageMap};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type MaxClaimLength: Get<u32>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		 Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		ClaimCreated(AccountId, Vec<u8>),
        ClaimRevoked(AccountId, Vec<u8>),
		ClaimTransfered(AccountId, Vec<u8>, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
        ProofAlreadyClaimed,
        NoSuchProof,
        NotProofOwner,
		ProofNotExists,
		ProofTooLong,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

         #[weight =0]
        fn create_claim(origin, proof: Vec<u8>) {
            let sender = ensure_signed(origin)?;

            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			ensure!( T::MaxClaimLength::get() >= proof.len() as u32, Error::<T>::ProofTooLong);

            let current_block = <frame_system::Module<T>>::block_number();

            Proofs::<T>::insert(&proof, (&sender, current_block));

            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }

        #[weight = 0]
        fn revoke_claim(origin, proof: Vec<u8>) {
            let sender = ensure_signed(origin)?;

            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            let (owner, _) = Proofs::<T>::get(&proof);

            ensure!(sender == owner, Error::<T>::NotProofOwner);

            Proofs::<T>::remove(&proof);

            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }

		#[weight = 0]
		pub fn transfer_claim(origin, proof: Vec<u8>, receiver: T::AccountId) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::ProofNotExists);
			let (owner, _block_number) = Proofs::<T>::get(&proof);
			ensure!( owner == sender , Error::<T>::NotProofOwner);
			Proofs::<T>::insert(&proof,(receiver.clone(),frame_system::Module::<T>::block_number()));
			Self::deposit_event(RawEvent::ClaimTransfered(sender, proof, receiver));
			Ok(())
		}

	}
}
