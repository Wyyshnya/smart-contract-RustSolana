use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::instruction::Instruction;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::system_instruction::create_account;
use solana_program_test::*;
use solana_sdk::account_info::IntoAccountInfo;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use spl_token::state::Account;
use sfxdx::{process_instruction};

#[tokio::test]
async fn test_initialize_store() {
    // Инициализация тестового окружения
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "your_program_name",
        id,
        processor!(process_instruction),
    )
    .start()
    .await;

    // Создание аккаунтов и ключей для магазина и токенов
    let store_account = Keypair::new();
    let token_account = Keypair::new();
    let owner_account = Keypair::new();

    // Инициализация магазина
    let store_data = Account {
        mint: token_account.pubkey(),
        owner: owner_account.pubkey(),
        amount: 1000,
        delegate: Default::default(),
        state: Default::default(),
        is_native: Default::default(),
        delegated_amount: 0,
        close_authority: Default::default(),
    };

    // Определение данных для инициализации аккаунта магазина
    let mut data = vec![];
    data.extend_from_slice(&store_data.is_initialized().to_le_bytes());
    data.extend_from_slice(&store_data.mint.to_bytes());
    data.extend_from_slice(&store_data.owner.to_bytes());
    data.extend_from_slice(&store_data.amount.to_le_bytes());

    // Создание транзакции для инициализации магазина
    let init_store_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };
    // Выполнение и ожидание транзакции
    let mut transaction = Transaction::new_with_payer(
        &[init_store_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Transaction failed: {:?}", result);

    // Получение аккаунта магазина и проверка его данных
    let store_account_info = next_account_info(&mut banks_client);
    let store_data = Account::unpack(&store_account_info.borrow()).unwrap();

    // Проверка, что магазин был успешно инициализирован
    assert_eq!(store_data.is_initialized(), true);
    assert_eq!(store_data.mint, token_account.pubkey());
    assert_eq!(store_data.owner, owner_account.pubkey());
    assert_eq!(store_data.amount, 1000);
}

#[tokio::test]
async fn test_update_price() {
    // Инициализация тестового окружения
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "your_program_name",
        id,
        processor!(process_instruction),
    )
    .start()
    .await;

    // Создание аккаунтов и ключей для магазина и токенов
    let store_account = Keypair::new();
    let token_account = Keypair::new();
    let owner_account = Keypair::new();

    // Инициализация магазина
    let store_data = Account {
        mint: token_account.pubkey(),
        owner: owner_account.pubkey(),
        amount: 1000,
        delegate: Default::default(),
        state: Default::default(),
        is_native: Default::default(),
        delegated_amount: 0,
        close_authority: Default::default(),
    };

    // Определение данных для инициализации аккаунта магазина
    let mut data = vec![];
    data.extend_from_slice(&store_data.is_initialized().to_le_bytes());
    data.extend_from_slice(&store_data.mint.to_bytes());
    data.extend_from_slice(&store_data.owner.to_bytes());
    data.extend_from_slice(&store_data.amount.to_le_bytes());

    // Создание транзакции для инициализации магазина
     let init_store_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };

    // Выполнение и ожидание транзакции и проверка успешной инициализации магазина
    let mut transaction = Transaction::new_with_payer(
        &[init_store_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Initialize Store transaction failed: {:?}", result);

    // Получение аккаунта магазина и проверка его данных
    let store_account_info = next_account_info(&mut banks_client,);
    let store_data = Account::unpack(&store_account_info.borrow()).unwrap();

    // Проверка, что магазин был успешно инициализирован
    assert_eq!(store_data.is_initialized(), true);
    assert_eq!(store_data.mint, token_account.pubkey());
    assert_eq!(store_data.owner, owner_account.pubkey());
    assert_eq!(store_data.amount, 1000);

    // Подготовка новой цены
    let new_price: u64 = 2000;
    let update_price_data = new_price.to_le_bytes();

    let mut data = vec![];
    data.extend_from_slice(&update_price_data);

     let update_price_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };

    // Выполнение и ожидание транзакции
    let mut transaction = Transaction::new_with_payer(
        &[update_price_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Update Price transaction failed: {:?}", result);

    // Получение аккаунта магазина и проверка обновленной цены
    let store_account_info = next_account_info(&mut banks_client);
    let store_data = Account::unpack(&store_account_info.borrow()).unwrap();

    // Проверка, что цена была успешно обновлена
    assert_eq!(store_data.amount, new_price);
}


#[tokio::test]
async fn test_sell() {
    // Инициализация тестового окружения
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "your_program_name",
        id,
        processor!(process_instruction()),
    )
    .start()
    .await;

    // Создание аккаунтов и ключей для магазина, токенов продавца и покупателя
    let store_account = Keypair::new();
    let seller_token_account = Keypair::new();
    let buyer_token_account = Keypair::new();
    let owner_account = Keypair::new();

    // Инициализация магазина
    let store_data = Account {
        mint: seller_token_account.pubkey(),
        owner: owner_account.pubkey(),
        amount: 1000,
        delegate: Default::default(),
        state: Default::default(),
        is_native: Default::default(),
        delegated_amount: 0,
        close_authority: Default::default(),
    };

    // Определение данных для инициализации аккаунта магазина
    let mut data = vec![];
    data.extend_from_slice(&store_data.is_initialized().to_le_bytes());
    data.extend_from_slice(&store_data.mint.to_bytes());
    data.extend_from_slice(&store_data.owner.to_bytes());
    data.extend_from_slice(&store_data.amount.to_le_bytes());

    // Создание транзакции для инициализации магазина
     let init_store_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), seller_token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };

    // Выполнение и ожидание транзакции и проверка успешной инициализации магазина
    let mut transaction = Transaction::new_with_payer(
        &[init_store_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &seller_token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Initialize Store transaction failed: {:?}", result);

    // Получение аккаунта магазина и проверка его данных
    let store_account_info = next_account_info(&mut banks_client);
    let store_data = Account::unpack(&store_account_info.borrow()).unwrap();

    // Проверка, что магазин был успешно инициализирован
    assert_eq!(store_data.is_initialized(), true);
    assert_eq!(store_data.mint, seller_token_account.pubkey());
    assert_eq!(store_data.owner, owner_account.pubkey());
    assert_eq!(store_data.amount, 1000); // Проверьте начальную цену

    // Подготовка токенов для продажи
    let seller_token_mint = Keypair::new();
    let seller_token_amount: u64 = 200; // Количество продаваемых токенов
    let buyer_token_mint = Keypair::new();

    let create_seller_token_mint_ix = spl_token::instruction::initialize_mint(
        &token_program_id,
        &seller_token_mint.pubkey(),
        &store_account.pubkey(),
        None,
        0,
    )
    .unwrap();

    let create_buyer_token_mint_ix = spl_token::instruction::initialize_mint(
        &token_program_id,
        &buyer_token_mint.pubkey(),
        &store_account.pubkey(),
        None,
        0,
    )
    .unwrap();

    let seller_create_token_account_ix = spl_token::instruction::initialize_account(
        &token_program_id,
        &seller_token_mint.pubkey(),
        &seller_token_account.pubkey(),
        &store_account.pubkey(),
    )
    .unwrap();

    let buyer_create_token_account_ix = spl_token::instruction::initialize_account(
        &token_program_id,
        &buyer_token_mint.pubkey(),
        &buyer_token_account.pubkey(),
        &store_account.pubkey(),
    )
    .unwrap();

    let mut data = vec![];
    data.extend_from_slice(&[0u8; 32]); // Заглушка для данных о токенах

    let create_seller_token_ix = create_account(
        &owner_account.pubkey(),
        &seller_token_account.pubkey(),
        1,
        10,
        &owner_account.pubkey());

    let create_buyer_token_ix = create_account(
        &owner_account.pubkey(),
        &buyer_token_account.pubkey(),
        1,
        10,
        &owner_account.pubkey()
    );

    // Отправка токенов на аккаунты продавца и покупателя
    let transfer_to_seller_ix = spl_token::instruction::transfer(
        &token_program_id,
        &owner_account.pubkey(),
        &seller_token_account.pubkey(),
        &store_account.pubkey(),
        &[],
        seller_token_amount,
    )
    .unwrap();

    // Создание транзакции для продажи
    let mut data = vec![];
    data.extend_from_slice(&seller_token_amount.to_le_bytes()); // Количество продаваемых токенов

    let sell_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), seller_token_account.pubkey(), buyer_token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };

    // Выполнение и ожидание транзакции
    let mut transaction = Transaction::new_with_payer(
        &[sell_ix, transfer_to_seller_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &seller_token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Sell transaction failed: {:?}", result);

    // Получение аккаунтов и проверка выполнения продажи
    let seller_account_info = next_account_info(&mut banks_client);
    let buyer_account_info = next_account_info(&mut banks_client);

    // Проверка, что продавец продал токены, а покупатель их получил
    let seller_token_account_data = spl_token::state::Account::unpack(&seller_account_info.borrow()).unwrap();
    let buyer_token_account_data = spl_token::state::Account::unpack(&buyer_account_info.borrow()).unwrap();

    assert_eq!(seller_token_account_data.amount, 0, "Seller still has tokens");
    assert_eq!(buyer_token_account_data.amount, seller_token_amount, "Buyer did not receive tokens");
}


#[tokio::test]
async fn test_buy() {
    // Инициализация тестового окружения
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "your_program_name",
        id,
        processor!(process_instruction),
    )
    .start()
    .await;

    // Создание аккаунтов и ключей для магазина, токенов продавца и покупателя
    let store_account = Keypair::new();
    let seller_token_account = Keypair::new();
    let buyer_token_account = Keypair::new();
    let owner_account = Keypair::new();

    // Инициализация магазина
    let store_data = Account {
        mint: seller_token_account.pubkey(),
        owner: owner_account.pubkey(),
        amount: 1000,
        delegate: Default::default(),
        state: Default::default(),
        is_native: Default::default(),
        delegated_amount: 0,
        close_authority: Default::default(),
    };

    // Определение данных для инициализации аккаунта магазина
    let mut data = vec![];
    data.extend_from_slice(&store_data.is_initialized().to_le_bytes());
    data.extend_from_slice(&store_data.mint.to_bytes());
    data.extend_from_slice(&store_data.owner.to_bytes());
    data.extend_from_slice(&store_data.amount.to_le_bytes());

    // Создание транзакции для инициализации магазина
     let init_store_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), seller_token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };
    // Выполнение и ожидание транзакции и проверка успешной инициализации магазина
    let mut transaction = Transaction::new_with_payer(
        &[init_store_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &seller_token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Initialize Store transaction failed: {:?}", result);

    // Получение аккаунтов и проверка успешной инициализации магазина
    let store_account_info = next_account_info(&mut banks_client);
    let store_data = Account::unpack(&store_account_info.borrow()).unwrap();

    // Проверка, что магазин был успешно инициализирован
    assert_eq!(store_data.is_initialized(), true);
    assert_eq!(store_data.mint, seller_token_account.pubkey());
    assert_eq!(store_data.owner, owner_account.pubkey());
    assert_eq!(store_data.amount, 1000);

    // Подготовка токенов для покупки
    let seller_token_mint = Keypair::new();
    let seller_token_amount: u64 = 100;
    let buyer_token_mint = Keypair::new();

    let create_seller_token_mint_ix = spl_token::instruction::initialize_mint(
        &token_program_id,
        &seller_token_mint.pubkey(),
        &store_account.pubkey(),
        None,
        0,
    )
    .unwrap();

    let create_buyer_token_mint_ix = spl_token::instruction::initialize_mint(
        &token_program_id,
        &buyer_token_mint.pubkey(),
        &store_account.pubkey(),
        None,
        0,
    )
    .unwrap();

    let seller_create_token_account_ix = spl_token::instruction::initialize_account(
        &token_program_id,
        &seller_token_account.pubkey(),
        &seller_token_mint.pubkey(),
        &store_account.pubkey(),
    )
    .unwrap();

    let buyer_create_token_account_ix = spl_token::instruction::initialize_account(
        &token_program_id,
        &buyer_token_account.pubkey(),
        &buyer_token_mint.pubkey(),
        &store_account.pubkey(),
    )
    .unwrap();

    let mut data = vec![];
    data.extend_from_slice(&[0u8; 32]); // Заглушка для данных о токенах

    let create_seller_token_ix = create_account(
        &owner_account.pubkey(),
        &seller_token_account.pubkey(),
        1,
        10,
        &owner_account.pubkey());

    let create_buyer_token_ix = create_account(&owner_account.pubkey(),
                                               &buyer_token_account.pubkey(),
                                               1,
                                               10,
                                               &owner_account.pubkey());

    // Отправка токенов на аккаунты продавца и покупателя
    let transfer_to_seller_ix = spl_token::instruction::transfer(
        &token_program_id,
        &owner_account.pubkey(),
        &seller_token_account.pubkey(),
        &owner_account.pubkey(),
        &[],
        seller_token_amount,
    )
    .unwrap();

    // Создание транзакции для покупки
    let buy_amount: u64 = 100; // Количество покупаемых токенов
    let mut data = vec![];
    data.extend_from_slice(&buy_amount.to_le_bytes()); // Количество покупаемых токенов

     let buy_ix = Instruction {
        program_id: id,
        accounts: [store_account.pubkey(), seller_token_account.pubkey(), buyer_token_account.pubkey(), owner_account.pubkey()].iter().map(|ac| ac.as_ref().unwrap()).collect(),
        data,
    };
    // Выполнение и ожидание транзакции
    let mut transaction = Transaction::new_with_payer(
        &[buy_ix, transfer_to_seller_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &store_account, &seller_token_account, &owner_account], recent_blockhash);

    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_ok(), "Buy transaction failed: {:?}", result);

    // Получение аккаунтов и проверка выполнения покупки
    let seller_account_info = next_account_info(&mut banks_client);
    let buyer_account_info = next_account_info(&mut banks_client);

    // Проверка, что продавец продал токены, а покупатель их получил
    let seller_token_account_data = spl_token::state::Account::unpack(&seller_account_info.borrow()).unwrap();
    let buyer_token_account_data = spl_token::state::Account::unpack(&buyer_account_info.borrow()).unwrap();

    assert_eq!(seller_token_account_data.amount, seller_token_amount - buy_amount, "Seller did not sell tokens");
    assert_eq!(buyer_token_account_data.amount, buy_amount, "Buyer did not receive tokens");
}