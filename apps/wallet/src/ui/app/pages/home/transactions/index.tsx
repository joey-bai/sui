// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import { memo, useEffect } from 'react';

import { Content } from '_app/shared/bottom-menu-layout';
import PageTitle from '_app/shared/page-title';
import TransactionCard from '_components/transactions-card';
import { useAppSelector, useAppDispatch } from '_hooks';
import { getTransactionsByAddress } from '_redux/slices/txresults';

import type { TxResultState } from '_redux/slices/txresults';

import st from './Transactions.module.scss';

function TransactionsPage() {
    const dispatch = useAppDispatch();
    const txByAddress: TxResultState[] = useAppSelector(
        ({ txresults }) => txresults.latestTx
    );

    useEffect(() => {
        dispatch(getTransactionsByAddress()).unwrap();
    }, [dispatch]);

    return txByAddress && txByAddress.length ? (
        <div className={st.container}>
            <PageTitle title="Your Activity" />
            <Content>
                <section className={st.txContent}>
                    {txByAddress.map((txn) => (
                        <TransactionCard txn={txn} key={txn.txId} />
                    ))}
                </section>
            </Content>
        </div>
    ) : null;
}

export default memo(TransactionsPage);
