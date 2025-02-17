//! # Scrypto Unit
//!
//! `scrypto_unit` is a lightweight testing framework for Scrypto.
//!
//! This crate contains a collection of useful methods that you can
//! leverage when testing your components.
#![allow(dead_code)]

extern crate radix_engine;
extern crate scrypto;

use radix_engine::engine::validate_data;
use radix_engine::ledger::SubstateStore;
use radix_engine::model::{Receipt, ValidatedInstruction};
use radix_engine::transaction::*;
use sbor::Decode;
use scrypto::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
/// The user account.
pub struct User {
    /// The user's public key.
    pub key: EcdsaPublicKey,
    /// The user's account address.
    pub account: Address,
}
/// Represents a test environment.
pub struct TestEnv<'a, L: SubstateStore> {
    /// The transaction executioner.
    pub executor: TransactionExecutor<'a, L>,
    /// The users of the test environment.
    pub users: HashMap<String, User>,
    /// The current user of the test environment.
    pub current_user: Option<User>,
    /// The test environment packages.
    pub packages: HashMap<String, Address>,
    /// The current package of the test environment.
    pub current_package: Option<Address>,
}

impl<'a, L: SubstateStore> TestEnv<'a, L> {
    /// Returns a test environment instance with the following fields:
    ///
    /// * `executor` - The transaction executioner.
    /// * `users` - The users of the test environment.
    /// * `current_user` - The current user of the test environment.
    /// * `packages` - The test environment packages.
    /// * `current_package` - The current package of the test environment.
    ///
    /// # Arguments
    ///
    /// * `ledger` - The transaction execution ledger.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    /// ```
    pub fn new(ledger: &'a mut L) -> Self {
        let executor = TransactionExecutor::new(ledger, false);
        let users: HashMap<String, User> = HashMap::new();
        let packages: HashMap<String, Address> = HashMap::new();

        Self {
            executor,
            users,
            current_user: None,
            packages,
            current_package: None,
        }
    }

    /// Returns a test environment instance exactly like `new` but with a tracing executor
    pub fn new_with_tracing(ledger: &'a mut L) -> Self {
        let executor = TransactionExecutor::new(ledger, true);
        let users: HashMap<String, User> = HashMap::new();
        let packages: HashMap<String, Address> = HashMap::new();

        Self {
            executor,
            users,
            current_user: None,
            packages,
            current_package: None,
        }
    }

    /// Publishes a given package to the transaction execution ledger.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package.
    /// * `package` - The package as a binary array.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    /// ```
    pub fn publish_package(&mut self, name: &str, package: &[u8]) -> &mut Self {
        let package_addr = self.executor.publish_package(package).unwrap();
        self.packages.insert(String::from(name), package_addr);

        //If first package set as default
        match self.current_package {
            Some(_) => {}
            None => self.current_package = Some(package_addr),
        }

        self
    }

    /// Retrieve a test environment package by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    ///
    /// let package = env.get_package("package");
    /// ```
    pub fn get_package(&self, name: &str) -> Address {
        match self.packages.get(name) {
            Some(&package) => package,
            None => panic!("No package named {:?} found.", name),
        }
    }

    /// Sets the current package of the test environment.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    ///
    /// env.using_package("package");
    /// ```
    pub fn using_package(&mut self, name: &str) -> &mut Self {
        let package = self.get_package(name);
        self.current_package = Some(package);

        self
    }

    /// Create a test user.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.create_user("test user");
    /// ```
    pub fn create_user(&mut self, name: &str) -> User {
        let key = self.executor.new_public_key();
        let account = self.executor.new_account(key);

        self.users.insert(String::from(name), User { key, account });

        let usr = User { key, account };

        //If first user set as default
        match self.current_user {
            Some(_) => {}
            None => self.current_user = Some(usr),
        }

        usr
    }

    /// Retrieve a test user by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.create_user("test user");
    ///
    /// let user = env.get_user("test user");
    /// ```
    pub fn get_user(&self, name: &str) -> &User {
        match self.users.get(name) {
            Some(user) => user,
            None => panic!("No user named {:?} found.", name),
        }
    }

    /// Set the current user of the test environment.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.create_user("test user");
    ///
    /// env.acting_as("test user");
    ///
    /// assert_eq!(env.get_current_user(), *env.get_user("test user"))
    /// ```
    pub fn acting_as(&mut self, name: &str) -> &mut Self {
        let user = self.get_user(name);
        self.current_user = Some(*user);

        self
    }

    /// Returns the current test user.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// let user = env.create_user("acc1");
    ///
    /// let current_user = env.get_current_user();
    ///
    /// assert_eq!(user, current_user);
    /// ```
    pub fn get_current_user(&self) -> User {
        match self.current_user {
            Some(user) => user,
            None => panic!("Fatal error, no user specified aborting"),
        }
    }

    /// Returns the current test package.
    ///
    /// # Examples
    ///
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    ///
    /// let current_package = env.get_current_package();
    /// ```
    pub fn get_current_package(&self) -> Address {
        match self.current_package {
            Some(package) => package,
            None => panic!("Fatal error, no package specified aborting"),
        }
    }

    /// Creates a token returns a ResourceDef
    /// # Arguments
    ///
    /// * `max_supply` - A decimal that defines the supply
    ///
    /// # Examples
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::InMemorySubstateStore;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    /// env.create_user("acc1");
    /// let token = env.create_token(10000.into());
    /// ```
    pub fn create_token(&mut self, max_supply: Decimal) -> ResourceDef {
        let user = self.get_current_user();
        let receipt = self
            .executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .new_token_fixed(HashMap::new(), max_supply.into())
                    .call_method_with_all_resources(user.account, "deposit_batch")
                    .build(vec![user.key])
                    .unwrap(),
            )
            .unwrap();

        return receipt.resource_def(0).unwrap().into();
    }

    /// Makes a function call and returns a Receipt
    /// # Arguments
    ///
    /// * `package_name`  - The name of the package as named in the blueprint
    /// * `function_name` - The name of the function to call
    /// * `params`        - A vector of Strings with the arguments to pass into the function
    ///
    /// # Examples
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    /// env.create_user("acc1");
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    /// let receipt = env.call_function("Hello", "new", vec!["1".to_owned()]);
    /// assert!(receipt.result.is_ok());
    /// ```
    pub fn call_function(
        &mut self,
        blueprint_name: &str,
        function_name: &str,
        params: Vec<String>,
    ) -> Receipt {
        let user = self.get_current_user();
        let package = self.get_current_package();
        self.executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .call_function(
                        package,
                        blueprint_name,
                        function_name,
                        params,
                        Some(user.account),
                    )
                    .call_method_with_all_resources(user.account, "deposit_batch")
                    .build(vec![user.key])
                    .unwrap(),
            )
            .unwrap()
    }

    /// Makes a method call and returns a Receipt
    /// # Arguments
    ///
    /// * `Component`   - A reference to the Address of the component
    /// * `method_name` - The name of the method
    /// * `params`      - A vector of Strings with the arguments to pass in the method
    ///
    /// # Examples
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// env.create_user("acc1");
    /// env.publish_package(
    ///     "package",
    ///     include_code!("../tests/assets/hello-world", "hello_world")
    /// );
    ///
    /// let receipt = env.call_function("Hello", "new", vec!["1".to_owned()]);
    /// assert!(receipt.result.is_ok());
    /// let component = receipt.component(0).unwrap();
    ///
    /// let receipt_method = env.call_method(
    ///     &component,
    ///     "update_state",
    ///     vec!["2".to_owned()]
    /// );
    /// assert!(receipt_method.result.is_ok());
    /// ```
    pub fn call_method(
        &mut self,
        component: &Address,
        method_name: &str,
        params: Vec<String>,
    ) -> Receipt {
        let user = self.get_current_user();

        self.executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .call_method(*component, method_name, params, Some(user.account))
                    .call_method_with_all_resources(user.account, "deposit_batch")
                    .build(vec![user.key])
                    .unwrap(),
            )
            .unwrap()
    }

    fn get_vault_info(ledger: &L, component_address: &Address, vid: &Vid) -> (Address, Contents) {
        let vault = ledger.get_vault(&component_address, vid).unwrap();

        let resource_def = ledger.get_resource_def(vault.resource_address()).unwrap();
        let contents = match resource_def.resource_type() {
            ResourceType::Fungible { .. } => Contents::Amount(vault.amount()),
            ResourceType::NonFungible => {
                Contents::NonFungibleKeys(vault.get_non_fungible_ids().unwrap())
            }
        };
        let resource_def_address = vault.resource_address();

        (resource_def_address, contents)
    }

    fn get_lazymap_info(
        ledger: &L,
        component_address: &Address,
        id: &Mid,
    ) -> Vec<(Address, Contents)> {
        let lazy_map = ledger.get_lazy_map(component_address, id).unwrap();
        lazy_map
            .map()
            .iter()
            .flat_map(|(_, data)| {
                let validated_data = validate_data(data).unwrap();
                validated_data
                    .vaults
                    .iter()
                    .map(|vid| TestEnv::get_vault_info(ledger, component_address, vid))
                    .collect::<Vec<(Address, Contents)>>()
            })
            .collect()
    }

    /// Returns the amount of the resource for the component/account
    /// # Arguments
    ///
    /// * `component_address`    - The Address of the component that holds the resource
    /// * `resource_def` - The Address that holds the resource
    ///
    /// # Examples
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::*;
    /// use scrypto::prelude::*;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    ///
    /// let user = env.create_user("acc1");
    /// let amount = env.get_amount_for_rd(user.account, RADIX_TOKEN);  
    /// assert!( amount == 1000000.into() );
    /// ```
    pub fn get_amount_for_rd(
        &mut self,
        component_address: Address,
        resource_def: Address,
    ) -> Decimal {
        let vaults = self.get_account_vaults(component_address);
        for (address, contents) in vaults {
            if address == resource_def {
                match contents {
                    Contents::Amount(amount) => return amount,
                    _ => panic!("Cannot get amount: resource {} is not fungible", address),
                }
            }
        }

        0.into()
    }


    pub fn get_non_fungible_keys_for_rd(&mut self, component_address: Address, resource_def: Address) -> Vec<NonFungibleKey> {
        let vaults = self.get_account_vaults(component_address);
        for (address, contents) in vaults {
            if address == resource_def {
                match contents {
                    Contents::NonFungibleKeys(keys) => return keys,
                    _ => panic!("Cannot get amount: resource {} is not fungible", address),
                }
            }
        }

        Vec::new()
    }
    
    pub fn get_account_vaults(&mut self, component_address: Address) -> HashMap<Address, Contents> {
        let ledger = self.executor.ledger();
        let component = ledger.get_component(component_address).unwrap();
        let state = component.state();
        let validated_data = validate_data(state).unwrap();
        validated_data
            .lazy_maps
            .iter()
            .flat_map(|mid| TestEnv::get_lazymap_info(ledger, &component_address, &mid))
            .collect()
    }

    /// Transfers some resource between users
    /// # Arguments
    ///
    /// * `amount` - A decimal that defines the amount to transfer
    /// * `resource_def` - The resource_def for the resource to transfer
    /// * `to_user` - the user receiving the amount of resource
    ///
    /// # Examples
    /// ```
    /// use scrypto_unit::*;
    /// use radix_engine::ledger::InMemorySubstateStore;
    ///
    /// let mut ledger = InMemorySubstateStore::with_bootstrap();
    /// let mut env = TestEnv::new(&mut ledger);
    /// env.create_user("user1");
    /// let token = env.create_token(10000.into());
    /// let user2 = env.create_user("user2");
    /// env.transfer_resource(10.into(), &token, &user2);
    /// ```
    pub fn transfer_resource(
        &mut self,
        amount: Decimal,
        resource_def: &ResourceDef,
        to_user: &User,
    ) -> Receipt {
        let user = self.get_current_user();
        let receipt = self
            .executor
            .run(
                TransactionBuilder::new(&self.executor)
                    .withdraw_from_account(
                        &Resource::Fungible {
                            amount,
                            resource_address: resource_def.address(),
                        },
                        user.account,
                    )
                    .call_method_with_all_resources(to_user.account, "deposit_batch")
                    .build(vec![user.key])
                    .unwrap(),
            )
            .unwrap();

        receipt
    }
}

pub enum Contents {
    Amount(Decimal),
    NonFungibleKeys(Vec<NonFungibleKey>),
}

/// Decodes the return value from a blueprint function within a transaction from the receipt
/// # Arguments
///
/// * `receipt`  - The name of the package as named in the blueprint
/// * `blueprint_name` - The name of the blueprint to search for the matching Instruction::CallFunction
///
/// NOTE: a custom built transaction may have more than one matching call.  This convenience
///       function may not work in such cases.
///
/// # Examples
/// ```
/// use scrypto_unit::*;
/// use radix_engine::ledger::*;
/// use scrypto::prelude::*;
///
/// let mut ledger = InMemorySubstateStore::with_bootstrap();
/// let mut env = TestEnv::new(&mut ledger);
///
/// env.publish_package(
///     "package",
///     include_code!("../tests/assets/hello-world", "hello_world")
/// );
///
/// env.create_user("test user");
/// env.acting_as("test user");
///
/// const BLUEPRINT: &str = "Hello";
/// let mut receipt = env.call_function(BLUEPRINT, "new", vec!["1".to_owned()]);
/// assert!(receipt.result.is_ok());
/// let ret: Component = return_of_call_function(&mut receipt, BLUEPRINT);
/// ```
pub fn return_of_call_function<T: Decode>(receipt: &mut Receipt, target_blueprint_name: &str) -> T {
    let instruction_index = receipt
        .transaction
        .instructions
        .iter()
        .position(|i| match i {
            ValidatedInstruction::CallFunction {
                ref blueprint_name, ..
            } if blueprint_name == target_blueprint_name => true,
            _ => false,
        })
        .unwrap();
    let encoded = receipt.outputs.swap_remove(instruction_index).raw;
    scrypto_decode(&encoded).unwrap()
}

/// Decodes the return value from a component method call within a transaction from the receipt
/// # Arguments
///
/// * `receipt`  - The name of the package as named in the blueprint
/// * `method_name` - The name of the method to search for the matching Instruction::CallMethod
///
/// NOTE: a custom built transaction may have more than one matching call.  This convenience
///       function may not work in such cases.
///
/// # Examples
/// ```
/// use scrypto_unit::*;
/// use radix_engine::ledger::*;
/// use scrypto::prelude::*;
///
/// let mut ledger = InMemorySubstateStore::with_bootstrap();
/// let mut env = TestEnv::new(&mut ledger);
///
/// env.publish_package(
///     "package",
///     include_code!("../tests/assets/hello-world", "hello_world")
/// );
///
/// env.create_user("test user");
/// env.acting_as("test user");
///
/// const BLUEPRINT: &str = "Hello";
/// let mut receipt = env.call_function(BLUEPRINT, "new", vec!["42".to_owned()]);
/// assert!(receipt.result.is_ok());
/// let component: Component = return_of_call_function(&mut receipt, BLUEPRINT);

/// let mut receipt = env.call_method(&component.address(), "update_state", vec!["77".to_owned()]);
/// assert!(receipt.result.is_ok());
/// let ret: u32 = return_of_call_method(&mut receipt, "update_state");
/// assert!(ret == 42);
/// ```
pub fn return_of_call_method<T: Decode>(receipt: &mut Receipt, method_name: &str) -> T {
    let instruction_index = receipt
        .transaction
        .instructions
        .iter()
        .position(|i| match i {
            ValidatedInstruction::CallMethod { ref method, .. } if method == method_name => true,
            _ => false,
        })
        .unwrap();
    let encoded = receipt.outputs.swap_remove(instruction_index).raw;
    scrypto_decode(&encoded).unwrap()
}
