# CAPE-OPEN Multi-Stream Heat Exchanger
This is a chemical process modelling component developed as a Dynamic-Link Library (DLL) in C++ using CAPE-OPEN Binary Interop Architecture (COBIA)
middleware. This makes it compatible with any CAPE-OPEN Standard compliant simulator. The unit aims to simulate a MultiStream Heat Exchanger.

## Definitions

### [Computer-Aided Process Engineering Open Standard (CAPE-OPEN)](https://www.colan.org/general-information-on-co-lan/)
> CAPE-OPEN consists of a series of specifications to expand the range of application of process simulation technologies. The CAPE-OPEN specifications specify a set of software interfaces that allow plug and play inter-operability between a given process modelling environment (PME) and a third-party process modelling component (PMC).
> CAPE-OPEN is an EU funded project supported by the non-profit organization [CO-LaN](https://www.colan.org/).

### [CAPE-OPEN Binary Interop Architecture (COBIA)](https://www.colan.org/experiences-projects/cape-open-binary-interop-architecture-cobia/)
> A new middleware, the CAPE-OPEN Binary Interop Architecture (COBIA), is the next step in the evolution of CAPE-OPEN. COBIA includes registration components, binary interoperability standards, and middleware that acts as a bridge between software components. Development of COBIA involves a number of tasks, grouped in phases, which are performed incrementally.
> COBIA serves as a propritary replacement to Microsoft's Component Object Model (COM), upon which all earlier developments have relied.


## Dependencies
1. [COBIA-Development SDK](https://colan.repositoryhosting.com/trac/colan_cobia/downloads) v1.2.0.8 
2. [Windows 11 SDK](https://developer.microsoft.com/en-us/windows/downloads/sdk-archive/) v10.0.22000.194

## Implementation
The unit accepts any number of inlet streams (currently set to 5). The first two streams are mandatory and must have different sides (cold/hot) while the rest of the streams are optional if their side is set to "Ignore". All inlet/outlet pairs must be both connected even if they are ignored or both disconnected.
### Input
1. Inlet 1~5 (material stream)
2. Inlet 1~5 Side (String Parameter)
### Output
1. Outlet 1~5 (material stream)

<img width="708" alt="image" src="https://user-images.githubusercontent.com/80135041/150345537-42616fb7-c41f-4de9-bbd6-7543c4527758.png">

## Compiling and registering
MSVC142 or equivalent. Regsiter using COBIA Developer Command Prompt `cobiaregister PATH-TO-MODULE`
