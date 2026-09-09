//! Product catalog: which QIUI toys map to which cloud actions + BLE profile.

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
        title_zh: "Cellmate 贞操锁 (Gen2/Gen3)",
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
        id: "keypod-metal",
        title_zh: "KeyPod Metal 金属钥匙舱",
        title_en: "KeyPod Metal",
        profile: Profile::Fee5,
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
        id: "keypod",
        title_zh: "KeyPod / KeyPod Pro",
        title_en: "KeyPod",
        profile: Profile::Fee5,
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
        id: "pearflower3",
        title_zh: "PearFlower Three / AnalPlug3",
        title_en: "PearFlower Three",
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
        id: "pearflower",
        title_zh: "PearFlower（旧款）",
        title_en: "PearFlower",
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
        id: "shake-metal",
        title_zh: "GenMetal / 震动金属锁",
        title_en: "Shake GenMetal lock",
        profile: Profile::Ae3,
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
        id: "metal-lock",
        title_zh: "Metal Lock 金属锁",
        title_en: "Metal Lock",
        profile: Profile::Ae3,
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
        title_zh: "PulseBird",
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
        id: "beatpat",
        title_zh: "BeatPat / StrikePad",
        title_en: "BeatPat",
        profile: Profile::Ac8,
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
        id: "collar",
        title_zh: "电击项圈",
        title_en: "Electric shock collar",
        // Collar UI lives under ProductActivity + electricShockRecord; BLE family
        // often shares fee5-class stacks. Prefer fee5 unless your device needs another.
        profile: Profile::Fee5,
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
];
