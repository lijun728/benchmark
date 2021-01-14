#![cfg_attr(not(feature = "std"), no_std)]

// use std::collections::hash_map::RandomState;
use frame_system::Module;
use frame_system::ensure_signed;

use codec::{Encode,Decode};
use frame_support::{decl_module,decl_storage, decl_event, decl_error, StorageValue, ensure, StorageMap, traits::Randomness, Parameter,traits::{Get, Currency, ReservableCurrency}
};
// use frame_support::{StorageMap, StorageValue, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::{Get, Randomness}};
use sp_io::hashing::blake2_128;
use sp_runtime::DispatchError;
use sp_runtime::traits::{AtLeast32Bit,Bounded};

//--------第二题  KittyIndex不在pallet中指定，而是在/runtime/src/lib.rs里面绑定--------- 
// type KittyIndex = u32;  

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode,Decode)]
pub  struct Kitty(pub [u8;16]);

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Parameter + AtLeast32Bit + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    type NewKittyReserve: Get<BalanceOf<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as KittiesModule {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;


        //--------第三题：扩展存储，能得到一个账号拥有的所有kitties--------BEGIN
        // 记录某个拥有者与猫之间的关系
        pub OwnedKitties get(fn owned_kitties):double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;
        // 记录某只猫的父母，因为猫可能没有父母，所以用 Option
        pub KittyParents get(fn kitty_parents):map hasher(blake2_128_concat) T::KittyIndex => Option<(T::KittyIndex, T::KittyIndex)>;
        // 记录某只猫的孩子们，第一个值是主猫，第二个是孩子，值也是孩子
        pub KittyChildren get(fn kitty_children):double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;
        // 记录某只猫的伴侣，第一个是主猫，第二个是伴侣猫，值是伴侣猫
        pub KittyPartners get(fn kitty_partners):double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<T::KittyIndex>;
        //--------------第三题-----------END
	}
}

decl_event!(
	pub enum Event<T> where AccountId=<T as frame_system::Trait>::AccountId, KittyIndex = <T as Trait>::KittyIndex {
        Created(AccountId, KittyIndex),
        Tansferred(AccountId, AccountId, KittyIndex),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        KittyNotExists,
		NotKittyOwner,
		TransferToSelf,
        RequiredDiffrentParent,
		MoneyNotEnough,
        UnReserveMoneyNotEnough,
        InvalidKittyId,
    }
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error =Error<T>;
        fn deposit_event() = default;



        #[weight = 0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex){
			let sender = ensure_signed(origin)?;
			// 判断 KittyIndex 是否存在，通过 ok_or 将错误抛出来，如果没有将返回一个 option 类型的数据
			let owner = Self::kitty_owner(kitty_id).ok_or( Error::<T>::KittyNotExists )?;
			// 判断 KittyIndex 是否属于发送者
			ensure!(owner == sender, Error::<T>::NotKittyOwner);

			// 不能转让给自己
			ensure!(to != sender, Error::<T>::TransferToSelf);
//-------第6题    在transfer的时候能转移质押   -----------BEGIN 
			// 质押被转让人的代币
			T::Currency::reserve(&to, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;

			// 解质押转出人的代币
			// 如果配置的质押代币数量变化了，可能这里会出问题。其实最好的方式是每个 kitty 都记录下，它当时质押的代币数量
			T::Currency::unreserve(&sender, T::NewKittyReserve::get());

			// 修改 KITTY 的拥有人
			KittyOwners::<T>::insert(kitty_id, &to);
			// 从之前的拥有人中删除关系
			OwnedKitties::<T>::remove(&sender, kitty_id);
			OwnedKitties::<T>::insert(&to, kitty_id, kitty_id);

			// 触发转让的事件
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
   //---------------第6题------------------END
		}

        #[weight = 0]
        pub fn breed( origin, kitty_id_1: T::KittyIndex, kitty_id_2:  T::KittyIndex ){
            let sender=ensure_signed(origin)?;
            let new_kitty_id=Self::do_breed(&sender, kitty_id_1,kitty_;id_2);
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));  
        }

        #[weight = 0]
        pub create(origin){
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let Kitty = Kitty(dna);
 //-------------第6题   create和breed需要质押一定数量的token
            T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;

            Self::insert_kitty( &sender, kitty_id, kitty, None );
            Self::deposit_event( RawEvent::Created(sender, kitty_id) );
        }


    }
}

fn combine_dna(dna1: u8, dna2 :u8, selector: u8 ) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T>{
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty, parent: Option<(T::KittyIndex, T::KittyIndex)>  ){
        <Kitties::<T>>::insert(kitty_id,kitty);
        <KittiesCount::<T>>::put(kitty_id+1.into());
        <KittyOwners::<T>>::insert(kitty_id, owner);
        <OwnedKitties::<T>>::insert(owner, kitty_id, kitty_id);
        match parent {
			Some((parent_id1, parent_id2)) =>{
				// 保存 kitty 的父母
				 <KittyParents::<T>>::insert(kitty_id, (parent_id1, parent_id2) );
				// 保存父母的孩子
				 <KittyChildren::<T>>::insert(parent_id1, kitty_id, kitty_id);
				 <KittyChildren::<T>>::insert(parent_id2, kitty_id, kitty_id);
				 // 保存父母的伴侣关系
				 <KittyPartners::<T>>::insert(parent_id1, parent_id2, parent_id2);
				 <KittyPartners::<T>>::insert(parent_id2, parent_id1, parent_id1);
			}
			_ => (),
		}

    }

    fn next_kitty_id()->sp_std::result::Result<T::KittyIndex,DispatchError>{
        let kitty_id = Self::kitties_count();
         if let kitty_id = T::KittyIndex::max_value(){
             return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId)->[u8; 16]{
        let payload=(
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

  

    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError>{
        let kitty1=Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2=Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?; 
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequiredDiffrentParent);

        let owner1 = Self::kitty_owner(kitty_id_1).ok_or( Error::<T>::KittyNotExists )?;
		let owner2 = Self::kitty_owner(kitty_id_2).ok_or( Error::<T>::KittyNotExists )?;
		ensure!(owner1 == *sender, Error::<T>::NotKittyOwner);
        ensure!(owner2 == *sender, Error::<T>::NotKittyOwner);
        
        let kitty_1 = Self::kitties(kitty_id_1).ok_or( Error::<T>::KittyNotExists )?;
		let kitty_2 = Self::kitties(kitty_id_2).ok_or( Error::<T>::KittyNotExists )?;

        let kitty_id = Self::next_kitty_id()?;

        let kitty1_dna =kitty1.0;
        let kitty2_dna =kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8;16];

        for i in 0..kitty1_dna.len(){
            new_dna[i]=combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
        let kitty = Kitty(new_dna);

		T::Currency::reserve(&sender, T::NewKittyReserve::get()).map_err(|_| Error::<T>::MoneyNotEnough )?;

		Self::insert_kitty(sender, kitty_id, kitty, Some((kitty_id_1, kitty_id_2)));

    }
}