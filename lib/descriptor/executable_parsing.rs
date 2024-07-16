use goblin::{elf::Elf, mach::Mach, pe::header::Header as PEHeader};
use tokio::time::Instant;

use super::{Arch, OS};

/**
    Tries to parse the contents of and executable file and
    return the OS and architecture it was compiled for.

    Currently supports ELF, Mach-O and PE formats.
*/
#[tracing::instrument(skip(binary_contents), level = "trace")]
pub fn parse_executable(binary_contents: impl AsRef<[u8]>) -> Option<(OS, Arch)> {
    let binary_contents = binary_contents.as_ref();

    // Prioritize parsing the current OS's format first since
    // it's the most likely to be correct, for most use cases.
    let start = Instant::now();

    #[cfg(target_os = "linux")]
    let opt = {
        parse_elf(binary_contents)
            .or_else(|| parse_mach(binary_contents))
            .or_else(|| parse_pe(binary_contents))
    };

    #[cfg(target_os = "macos")]
    let opt = {
        parse_mach(binary_contents)
            .or_else(|| parse_elf(binary_contents))
            .or_else(|| parse_pe(binary_contents))
    };

    #[cfg(target_os = "windows")]
    let opt = {
        parse_pe(binary_contents)
            .or_else(|| parse_elf(binary_contents))
            .or_else(|| parse_mach(binary_contents))
    };

    tracing::trace!(
        elapsed = ?start.elapsed(),
        "parsed executable format"
    );

    opt
}

fn parse_elf(binary_contents: &[u8]) -> Option<(OS, Arch)> {
    Elf::parse_header(binary_contents).ok().and_then(|head| {
        use goblin::elf::header::{EM_386, EM_AARCH64, EM_ARM, EM_X86_64};

        let arch = match head.e_machine {
            EM_AARCH64 => Arch::Arm64,
            EM_X86_64 => Arch::X64,
            EM_386 => Arch::X86,
            EM_ARM => Arch::Arm32,
            _ => return None,
        };

        Some((OS::Linux, arch))
    })
}

fn parse_mach(binary_contents: &[u8]) -> Option<(OS, Arch)> {
    use goblin::mach::{
        constants::cputype::{
            CPU_TYPE_ARM, CPU_TYPE_ARM64, CPU_TYPE_ARM64_32, CPU_TYPE_X86, CPU_TYPE_X86_64,
        },
        cputype::CpuType,
    };

    #[allow(clippy::inline_always)]
    #[inline(always)]
    const fn cputype_to_arch(cputype: CpuType) -> Option<Arch> {
        match cputype {
            CPU_TYPE_ARM64 => Some(Arch::Arm64),
            CPU_TYPE_X86_64 => Some(Arch::X64),
            CPU_TYPE_ARM64_32 | CPU_TYPE_ARM => Some(Arch::Arm32),
            CPU_TYPE_X86 => Some(Arch::X86),
            _ => None,
        }
    }

    match Mach::parse(binary_contents).ok()? {
        Mach::Binary(macho) => {
            let arch = cputype_to_arch(macho.header.cputype())?;
            Some((OS::MacOS, arch))
        }
        Mach::Fat(fat) => {
            let arches = fat.arches().ok()?;
            let arches = arches
                .iter()
                .filter_map(|arch| cputype_to_arch(arch.cputype()))
                .collect::<Vec<_>>();
            if arches.is_empty() {
                None
            } else if arches.len() == 1 {
                Some((OS::MacOS, arches[0]))
            } else {
                // FUTURE: Handle multiple architectures / universal
                // binaries in Arch enum and propagate results here
                None
            }
        }
    }
}

fn parse_pe(binary_contents: &[u8]) -> Option<(OS, Arch)> {
    PEHeader::parse(binary_contents).ok().and_then(|header| {
        use goblin::pe::header::{
            COFF_MACHINE_ARM, COFF_MACHINE_ARM64, COFF_MACHINE_ARMNT, COFF_MACHINE_X86,
            COFF_MACHINE_X86_64,
        };

        let arch = match header.coff_header.machine {
            COFF_MACHINE_ARM64 => Arch::Arm64,
            COFF_MACHINE_X86_64 => Arch::X64,
            COFF_MACHINE_ARM | COFF_MACHINE_ARMNT => Arch::Arm32,
            COFF_MACHINE_X86 => Arch::X86,
            _ => return None,
        };

        Some((OS::Windows, arch))
    })
}
