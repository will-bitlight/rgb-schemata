// RGB schemata by LNP/BP Standards Association
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2023-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2023-2024 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Non-Inflatable Assets (NIA) schema implementing RGB20 fungible assets
//! interface.

use aluvm::isa::Instr;
use aluvm::library::{Lib, LibSite};
use chrono::Utc;
use ifaces::{rgb20, IfaceWrapper, IssuerWrapper, Rgb20, LNPBP_IDENTITY};
use rgbstd::interface::{IfaceImpl, NamedField, VerNo};
use rgbstd::schema::{
    FungibleType, GenesisSchema, GlobalStateSchema, Occurrences, OwnedStateSchema, Schema,
    TransitionSchema,
};
use rgbstd::stl::StandardTypes;
use rgbstd::validation::Scripts;
use rgbstd::vm::opcodes::{INSTR_PCCS, INSTR_PCVS};
use rgbstd::vm::RgbIsa;
use rgbstd::{rgbasm, Identity};
use strict_types::TypeSystem;

use crate::{GS_ISSUED_SUPPLY, GS_NOMINAL, GS_TERMS, OS_ASSET, TS_TRANSFER};

pub(crate) fn nia_lib() -> Lib {
    let code = rgbasm! {
        // SUBROUTINE 1: genesis validation
        // Checking pedersen commitments against reported amount of issued assets present in the
        // global state.
        pccs    0x0FA0,0x07D2   ;
        // If the check succeeds we need to terminate the subroutine.
        ret                     ;
        // SUBROUTINE 2: transfer validation
        // Checking that the sum of pedersen commitments in inputs is equal to the sum in outputs.
        pcvs    0x0FA0          ;
    };
    Lib::assemble::<Instr<RgbIsa>>(&code).expect("wrong non-inflatable asset script")
}
pub(crate) const FN_GENESIS_OFFSET: u16 = 0;
pub(crate) const FN_TRANSFER_OFFSET: u16 = 5 + 1;

fn nia_schema() -> Schema {
    let types = StandardTypes::with(Rgb20::stl());

    let alu_lib = nia_lib();
    let alu_id = alu_lib.id();
    assert_eq!(alu_lib.code.as_ref()[FN_GENESIS_OFFSET as usize], INSTR_PCCS);
    assert_eq!(alu_lib.code.as_ref()[FN_TRANSFER_OFFSET as usize], INSTR_PCVS);

    Schema {
        ffv: zero!(),
        flags: none!(),
        name: tn!("NonInflatableAsset"),
        developer: Identity::from(LNPBP_IDENTITY),
        meta_types: none!(),
        global_types: tiny_bmap! {
            GS_NOMINAL => GlobalStateSchema::once(types.get("RGBContract.AssetSpec")),
            GS_TERMS => GlobalStateSchema::once(types.get("RGBContract.AssetTerms")),
            GS_ISSUED_SUPPLY => GlobalStateSchema::once(types.get("RGBContract.Amount")),
        },
        owned_types: tiny_bmap! {
            OS_ASSET => OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
        },
        valency_types: none!(),
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                GS_NOMINAL => Occurrences::Once,
                GS_TERMS => Occurrences::Once,
                GS_ISSUED_SUPPLY => Occurrences::Once,
            },
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            valencies: none!(),
            validator: Some(LibSite::with(FN_GENESIS_OFFSET, alu_id)),
        },
        extensions: none!(),
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionSchema {
            metadata: none!(),
                globals: none!(),
                inputs: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                assignments: tiny_bmap! {
                    OS_ASSET => Occurrences::OnceOrMore
                },
                valencies: none!(),
                validator: Some(LibSite::with(FN_TRANSFER_OFFSET, alu_id))
            }
        },
    }
}

fn nia_rgb20() -> IfaceImpl {
    let schema = nia_schema();
    let iface = Rgb20::iface(rgb20::Features::NONE);

    IfaceImpl {
        version: VerNo::V1,
        schema_id: schema.schema_id(),
        iface_id: iface.iface_id(),
        timestamp: Utc::now().timestamp(),
        developer: Identity::from(LNPBP_IDENTITY),
        global_state: tiny_bset! {
            NamedField::with(GS_NOMINAL, fname!("spec")),
            NamedField::with(GS_TERMS, fname!("terms")),
            NamedField::with(GS_ISSUED_SUPPLY, fname!("issuedSupply")),
        },
        assignments: tiny_bset! {
            NamedField::with(OS_ASSET, fname!("assetOwner")),
        },
        valencies: none!(),
        transitions: tiny_bset! {
            NamedField::with(TS_TRANSFER, fname!("transfer")),
        },
        extensions: none!(),
    }
}

pub struct NonInflatableAsset;

impl IssuerWrapper for NonInflatableAsset {
    const FEATURES: rgb20::Features = rgb20::Features::NONE;
    type IssuingIface = Rgb20;

    fn schema() -> Schema { nia_schema() }
    fn issue_impl() -> IfaceImpl { nia_rgb20() }

    fn types() -> TypeSystem { StandardTypes::with(Rgb20::stl()).type_system() }

    fn scripts() -> Scripts {
        let lib = nia_lib();
        confined_bmap! { lib.id() => lib }
    }
}
