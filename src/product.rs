//! Product catalog: QIUI toys → cloud actions + default BLE profile.
//! typeId / UUID mapping follows app `SelectEquipmentActivity` + Feign prefixes.

use crate::protocol::Profile;

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub name: &'static str,
    pub summary: &'static str,
    pub path: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Product {
    pub id: &'static str,
    pub title_zh: &'static str,
    pub title_en: &'static str,
    pub profile: Profile,
    pub actions: &'static [Action],
}

pub fn all() -> &'static [Product] {
    PRODUCTS
}

pub fn get(id: &str) -> Option<&'static Product> {
    all().iter().find(|p| p.id == id)
}

pub fn find_action(product: &Product, action: &str) -> Option<&'static Action> {
    product.actions.iter().find(|a| a.name == action)
}

macro_rules! act {
    ($name:literal, $summary:literal, $path:literal) => {
        Action {
            name: $name,
            summary: $summary,
            path: $path,
        }
    };
}

static PRODUCTS: &[Product] = &[
    Product {
        id: "cellmate",
        title_zh: "Cellmate 贞操锁 (Gen2/Gen3，type 1/10)",
        title_en: "Cellmate lock",
        profile: Profile::Cellmate,
        actions: &[
            act!(
                "token",
                "会话 token（开写前）",
                "/feign/toyCellmateBluetooth/getToyToken"
            ),
            act!(
                "close-lock",
                "关锁",
                "/feign/toyCellmateBluetooth/toyCloseLock"
            ),
            act!(
                "shock",
                "立即电击",
                "/feign/toyCellmateBluetooth/getToyShockImmediately"
            ),
            act!(
                "decry",
                "解密设备 notify hex",
                "/feign/toyCellmateBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "collar",
        title_zh: "小恶魔 / 电击项圈 (type 3)",
        title_en: "Little Devil / electric collar",
        profile: Profile::Collar,
        actions: &[
            act!(
                "unlock",
                "云端处理开锁/电击相关命令（需 --hex）",
                "/feign/electricShockRecord/unlockToy"
            ),
            act!(
                "decrypt",
                "云端解密蓝牙命令（需 --hex）",
                "/feign/electricShockRecord/decryptToy"
            ),
        ],
    },
    Product {
        id: "keypod",
        title_zh: "钥匙盒 KeyPod (type 6)",
        title_en: "KeyPod",
        profile: Profile::KeyPod,
        actions: &[
            act!("token", "会话 token", "/feign/toyKeyPodBluetooth/getToyToken"),
            act!("lock", "上锁", "/feign/toyKeyPodBluetooth/toyLock"),
            act!("unlock", "开锁", "/feign/toyKeyPodBluetooth/toyUnlock"),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyKeyPodBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "keypod2",
        title_zh: "二代钥匙盒 KeyPod2 (type 11，GATT 同 Cellmate)",
        title_en: "KeyPod Pro/2",
        profile: Profile::Cellmate,
        actions: &[
            act!("token", "会话 token", "/feign/toyKeyPodBluetooth/getToyToken"),
            act!("lock", "上锁", "/feign/toyKeyPodBluetooth/toyLock"),
            act!("unlock", "开锁", "/feign/toyKeyPodBluetooth/toyUnlock"),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyKeyPodBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "keypod-metal",
        title_zh: "金属钥匙盒 KeyPod Metal (type 20→0b30；默认也可 fee5)",
        title_en: "KeyPod Metal",
        profile: Profile::KeyPodMetal20,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toyKeyPodMetalBluetooth/getToyToken"
            ),
            act!("lock", "上锁", "/feign/toyKeyPodMetalBluetooth/toyLock"),
            act!(
                "unlock",
                "开锁",
                "/feign/toyKeyPodMetalBluetooth/toyUnlock"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyKeyPodMetalBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "pearflower",
        title_zh: "梨花 / 二代肛塞 (type 9)",
        title_en: "PearFlower / AnalPlug2",
        profile: Profile::PearFlower3,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toyPearflowerBluetooth/getToyToken"
            ),
            act!(
                "shock",
                "立即电击",
                "/feign/toyPearflowerBluetooth/getInstantShock"
            ),
            act!(
                "jitter",
                "立即抖动",
                "/feign/toyPearflowerBluetooth/getInstantJitter"
            ),
            act!("stop", "全部停止", "/feign/toyPearflowerBluetooth/getStopAll"),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyPearflowerBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "pearflower3",
        title_zh: "三代肛塞 PearFlower Three (type 18)",
        title_en: "AnalPlug3 / PearFlower Three",
        profile: Profile::PearFlower3,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/pearFlowerThreeBluetooth/getToyToken"
            ),
            act!(
                "lock",
                "关锁",
                "/feign/pearFlowerThreeBluetooth/getPearFlowerThreeToyLock"
            ),
            act!(
                "unlock",
                "开锁",
                "/feign/pearFlowerThreeBluetooth/getFlowerThreeToyUnlock"
            ),
            act!(
                "shock",
                "电击",
                "/feign/pearFlowerThreeBluetooth/getPearFlowerThreeToyElectric"
            ),
            act!(
                "vibrate",
                "振动",
                "/feign/pearFlowerThreeBluetooth/getPearFlowerThreeToyVibration"
            ),
            act!(
                "stop-shock",
                "停止电击",
                "/feign/pearFlowerThreeBluetooth/getPearFlowerThreeToyCancelElectric"
            ),
            act!(
                "stop-vibrate",
                "停止振动",
                "/feign/pearFlowerThreeBluetooth/getPearFlowerThreeToyCancelVibration"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/pearFlowerThreeBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "tail",
        title_zh: "尾巴 Tail (type 12)",
        title_en: "Tail",
        profile: Profile::Fee5,
        actions: &[
            act!("token", "会话 token", "/feign/toyTailBluetooth/getToyToken"),
            act!(
                "sway-long",
                "长摇摆",
                "/feign/toyTailBluetooth/getLongSwayModelCmd"
            ),
            act!(
                "sway-line",
                "线性摇摆",
                "/feign/toyTailBluetooth/getLineSwayModelCommand"
            ),
            act!(
                "sway-heart",
                "心形摇摆",
                "/feign/toyTailBluetooth/getHeartSwayModelCmd"
            ),
            act!(
                "stop",
                "停止摇摆/电击",
                "/feign/toyTailBluetooth/getStopAllShakeAndElectricShockCommand"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyTailBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "metal-lock",
        title_zh: "金属锁 GenMetal (type 13)",
        title_en: "GenMetal / Metal Lock",
        profile: Profile::Fee5,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toyMetalLockBluetooth/getMetalLockTokenCmd"
            ),
            act!(
                "lock",
                "上锁",
                "/feign/toyMetalLockBluetooth/getMetalLockCmd"
            ),
            act!(
                "unlock",
                "开锁",
                "/feign/toyMetalLockBluetooth/getMetalLockUnLockCmd"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyMetalLockBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "pulsebird",
        title_zh: "脉冲鸟 PulseBird (type 14，GATT=fee5)",
        title_en: "PulseBird",
        profile: Profile::Fee5,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/oem/pulseBird/generatePulseBirdToken"
            ),
            act!(
                "decry",
                "解密结果",
                "/feign/oem/pulseBird/getPulseBirdDecryptResult"
            ),
        ],
    },
    Product {
        id: "shake-metal",
        title_zh: "震动金属锁 ShockGenMetal (type 15，GATT=fee5)",
        title_en: "Shock GenMetal",
        profile: Profile::Fee5,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toyShakeMetalLockBluetooth/getMetalLockTokenCmd"
            ),
            act!(
                "lock",
                "上锁",
                "/feign/toyShakeMetalLockBluetooth/getShakeMetalLockCmd"
            ),
            act!(
                "unlock",
                "开锁",
                "/feign/toyShakeMetalLockBluetooth/getShakeMetalLockUnLockCmd"
            ),
            act!(
                "shake",
                "立即震动",
                "/feign/toyShakeMetalLockBluetooth/getShakeMetalLockImmediatelyShakeCmd"
            ),
            act!(
                "stop",
                "停止震动/电击",
                "/feign/toyShakeMetalLockBluetooth/stopAllShakeAndElectricCmd"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyShakeMetalLockBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "beatpat",
        title_zh: "电击板 StrikePad / BeatPat (type 5；云端有命令)",
        title_en: "StrikePad / BeatPat",
        // SelectEquipment clears GATT UUIDs for type 5; use fee5 only as a soft default for write.
        profile: Profile::Fee5,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toyBeatPatBluetooth/getToyToken"
            ),
            act!(
                "strength",
                "开强度",
                "/feign/toyBeatPatBluetooth/getNewStrength"
            ),
            act!(
                "strength-off",
                "关强度",
                "/feign/toyBeatPatBluetooth/getNewStrengthClose"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toyBeatPatBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "femboy",
        title_zh: "Femboy / SissyStar 锁 (type 19)",
        title_en: "Femboy / SissyStar",
        profile: Profile::Fee5,
        actions: &[
            act!(
                "token",
                "会话 token",
                "/feign/toySissyStarLockBluetooth/getSissyStarLockTokenCmd"
            ),
            act!(
                "lock",
                "上锁",
                "/feign/toySissyStarLockBluetooth/getSissyStarLockCmd"
            ),
            act!(
                "unlock",
                "开锁",
                "/feign/toySissyStarLockBluetooth/getSissyStarUnLockCmd"
            ),
            act!(
                "decry",
                "解密 notify",
                "/feign/toySissyStarLockBluetooth/decryBluetoothCommand"
            ),
        ],
    },
    Product {
        id: "masturbator",
        title_zh: "飞机杯 Masturbator (type 16；多为 Web/本地控，云端 hex 少)",
        title_en: "Masturbator / airplane cup",
        profile: Profile::Ac8,
        actions: &[act!(
            "disconnect",
            "App 断开玩具",
            "/feign/airplane_cup/appDisconnectToy"
        )],
    },
];
