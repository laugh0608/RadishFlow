---
title: Manager Common Interface Specification
---
## Summary

This document contains the textual CAPE-OPEN specification of the CAPE-OPEN Primary Process Modelling Component (PMC) called Manager that creates and manages a set of Primary PMC objects belonging all to the same category of Primary PMCs. Examples of Managers are Property Package Managers and Chemical Reaction Package Managers. The concept is extendable to Managers of other Primary PMC objects such as Unit Operations.

## Acknowledgements

This CAPE-OPEN Interface Specification Document has been developed within the Thermo Special Interest Group. The following are the main contributors:

* Jasper van Baten AmsterCHEM
* Sergej Blagov BASF SE
* Michel Pons CO-LaN
* Klaus Möller University of Cape Town

## Introduction

The interface specification of a Manager is a generalization of the specification of a Property Package Manager (i). However, the Property Package Manager specification has some shortcomings, in particular for the configuration of new Property Packages and the restoration of Property Packages from persistence.

This document introduces the concept of a Manager that is not specific to Property Packages, while addressing the above shortcomings.

### Document Roadmap

This document is a textual CAPE-OPEN Interface Specification Document of the CAPE-OPEN Primary Process Modelling Component (PMC) called Manager that creates and manages a set of Primary PMC Objects (vii) belonging all to the same category of Primary PMCs. A Manager allows creation of a Primary PMC Object by reloading a previously saved state, by selecting from a list of pre-configured Templates, or from scratch as a new unconfigured Primary PMC Object.

The document belongs to the documentation set of CAPE-OPEN interface specifications in its version 1.1. This imposes that for software components implementing the interface defined in this document, the CapeVersion registration value must be set to “1.1”. 

### Audience  

This document is intended primarily for software developers who want to create CAPE-OPEN Managers.

For any reader, an understanding of UML diagrams is assumed.

This document is not intended for Process Engineers who are end-users of CAPE-OPEN process simulation components or process environments.

## Requirements

This chapter defines the overall architectural design, the textual requirements on PMEs and Managers, and Use Cases.

### Architecture

Any Primary PMC Object is characterized by a component type. The characterization is made through a pre-defined Category ID for stand-alone Primary PMC Objects or through the Category ID of the Manager that creates the Primary PMC Object. For example, there is a pre-defined Category ID of a stand-alone Property Package and there is a pre-defined Category ID for a Property Package Manager.

Some PMCs are custom-made for one scenario and have only one configuration. In that case these PMCs will be made stand-alone PMCs by their developer/vendor. In other cases, a more generic software component allows for customization of PMC objects by the end-user. For example, a Property Package Manager allows for configuration of a Property Package (e.g., selection of compounds, of thermodynamic models, of phases supported). Instead of requiring each of the configurations to be registered as a stand-alone software component, a Manager is the single software component to be registered, and visible to the end-user as such.

Each Manager software component type has a stand-alone counterpart. For example, a Manager for Chemical Reaction Packages, a.k.a. a Chemical Reaction Package Manager, finds its counterpart in a stand-alone Chemical Reaction Package, which is not managed by a Manager. This document does not describe how any stand-alone PMC Object behaves.

A Manager is responsible for instantiating Primary PMC Objects. A Manager maintains a set of Templates, each containing the configuration of the instance of a Primary PMC Object. There are three different scenarios leading to the instantiation of a Primary PMC Object. A PME can request a Manager to create Primary PMC Objects as follows:

1. an un-configured Primary PMC Object for subsequent de-persistence by the PME,

2. from a list of pre-configured Templates,

3. a new PMC Object to be configured on-the-fly by the Flowsheet Builder.

In all circumstances, the remaining lifespan of a Primary PMC object created by a Manager is like for any Primary PMC object, starting with *ICapeUtilities::Initialize* and ending with *ICapeUtilities::Terminate*.

A Manager is a Primary PMC Object. Primary PMC Objects may implement persistence. However, a Manager should not implement persistence and a PME must not persist a Manager. The configuration of a Manager consists of a list of Templates and the configuration of each Template. Indeed, the configuration of the Manager is typically defined at the machine or current user level, not at the flowsheet level. 

The visibility of a Template is not restricted to a particular session of the Manager or to a particular PME. Once a Template is created, it is visible in all PMEs through the Manager until the Manager deletes the Template. The way Templates are stored by the Manager is internal to the Manager.

To ensure portability over time and between machines, there is a need for a straightforward workflow that requires persistence to be implemented on Primary PMC Objects created through a Manager. Therefore, Primary PMC Objects created by the Manager must implement persistence. 

The workflow for restoring a Primary PMC is distinctly different from the workflow for creating a new Primary PMC that is not based on a Template. In contrast to functionality offered by *ICapeThermoPropertyPackageManager* (i), for restoring a Primary PMC, the PME does not need to pick a Template name from a list and specify it to the Manager. Instead of the PME saving the Template name, the PME calls a method on the Manager that creates and returns the Primary PMC object for the purpose of de-persistence. Subsequently the PME de-persists the returned Primary PMC Object.

A Manager advertises whether it supports creating a new instance from scratch.

As a Primary PMC, a Manager and the objects it creates implement *ICapeUtilities* and therefore may support editing functionality through *ICapeUtilities::Edit*. If a component created by a Manager is being edited, the expectation is to change the configuration of the component. However, editing a Manager translates into managing its list of Templates (e.g., add, remove, rename, edit a Template). A Manager may provide functionality to save, as a Template, the configuration of a particular instance of the Primary PMC Object created by the Manager, for example via a button in the GUI of the Primary PMC Object. Such functionality would allow the end-user to save a Template in the form of an instance in the flowsheet document to transfer the Template to another machine. The Manager may also provide more direct functionality to import/export Templates, for example through the private GUI of the Manager. All the above operations that pertain to the configuration of the Manager are private to the Manager and fall outside of the scope of this interface specification.

In the scope of the Manager, a Template is identified through its name only. In the current version of the specification, no other generic descriptor of a Template is provided.

Through the Manager there is no access to internal information of the Template or the Primary PMCs corresponding to the Template. If more information is needed to select a  template, an instance of a Primary PMC Object must be created from the Template by the Manager so that this additional information may be retrieved from that instance itself. The Primary PMC Object should not be instantiated unnecessarily because it can be costly and prone to errors.

### Textual requirements

Through textual requirements, this section explains how a Manager will interact with other software such as a PME or a Primary Process Modelling Component. These functional requirements are mostly written in natural language with technical terms defined in the Glossary when used repeatedly. Requirements are not organized in order of importance,
urgency, and convenience.

#### REQ-MGR-01: A Manager is a primary Process Modelling Component. {#req-mgr-01}

*<u>Rationale:</u>* A Manager is created by a PME so it is a Primary Process Modelling Component.

#### REQ-MGR-02: A Manager is categorized as a CAPE-OPEN component. {#req-mgr-02}

*<u>Rationale:</u>* The Category ID for a CAPE-OPEN component appears in the registry for each of the CAPE-OPEN Primary PMCs. In future CAPE-OPEN versions, the ”CAPE-OPEN Component” Category ID will be used to indicate that CAPE-OPEN version 1.1 is supported by the component.

#### REQ-MGR-03: Objects instantiated by a Manager are Primary Process Modelling Components. {#req-mgr-03}

*<u>Rationale</u>*: After instantiation by a Manager, from the PME point of view, the Primary PMC Object is indistinguishable from a stand-alone Primary PMC Object of the same type.

#### REQ-MGR-04: A Manager is registered with CAPE-OPEN Category ID(s) indicating which type(s) of software components the Manager creates. {#req-mgr-04}

*<u>Rationale:</u>* A Manager is a Primary Process Modelling Component. Each Primary Process Modelling Component belongs to a category that allows a PME to create a list of the Primary Process Modelling Components belonging to this category without the need to instantiate each of those.

Without the Manager indicating the type(s) of Manager it is, the PME would list Property Package Managers and Chemical Reaction Package Managers without distinguishing between them. There can be no Manager without knowledge of which sort of Templates it manages.

Only stand-alone Primary PMC Objects are registered with their own Category IDs.

#### REQ-MGR-05: Each instance created by a Manager is of the software component type(s) consistent with the Manager type(s). {#req-mgr-05}

*<u>Rationale:</u>* For example, if the Manager is a Property Package Manager, all software components created by the Manager must be Property Packages and must comply with the requirements on a Property Package as outlined in (i).

If the Manager is a Chemical Reaction Package Manager, all software components created by the Manager must be Chemical Reaction Packages and must comply with the requirements on a Chemical Package as outlined in (ii).

If the Manager is both a Property Package Manager and a Chemical Reaction Package Manager, all software components created by the Manager must be Property Packages supporting Chemical Reactions and must comply with the requirements on a Property Package supporting Chemical Reactions as outlined in (ii).

#### REQ-MGR-06: A Manager provides the service to deliver the list of names of its Templates. {#req-mgr-06}

*<u>Rationale:</u>* The name of the Template is needed to request the creation of a Primary PMC based on the Template. For ease of interaction with the Process Engineer, the PME must have access to the list of Templates supported by the Manager.

Note that it is not required from the Manager to be able to create Primary PMC Objects from all the Templates listed. Creation may fail for a number of reasons (e.g., ill configured Template, license is not present for a particular feature).

#### REQ-MGR-07: A Manager creates an instance from a Template with the same name as the Template. {#req-mgr-07}

*<u>Rationale:</u>* Matter of consistency which can be helpful to the Process Engineer. If names need to be unique (names used in a Collection for instance), it is the responsibility of the PME to rename the instance as needed using operations from *ICapeIdentification* (x). 

#### REQ-MGR-08: The name of a Template follows the rules of the name property in *ICapeIdentification.* {#req-mgr-08}

*<u>Rationale:</u>* Since the instance created from a Template is identified by the name of the Template, the name of the Template must follow the rules applicable to any name identifying a CAPE-OPEN component through *ICapeIdentification* (x).

#### REQ-MGR-09: A Template is identified by its name in a case insensitive manner.  {#req-mgr-09}

*<u>Rationale:</u>* Case insensitiveness is in accordance with choices made in CAPE-OPEN.

#### REQ-MGR-10: All Template names must be unique within a Manager. {#req-mgr-10}

*<u>Rationale:</u>* Names are used by the Manager and the PME to uniquely identify Templates.

#### REQ-MGR-11: If a PME persists the state of a Primary PMC Object created via a Manager, the PME persists also sufficient information to create the Manager. {#req-mgr-11}

*<u>Rationale:</u>* The PME cannot directly instantiate a Primary PMC Object originally created via a Manager. Therefore, the PME must be able to find which Manager needs to be instantiated in order for the proper Manager to load back the Primary PMC Object. The PME needs to keep sufficient information (e.g., ProgID, version independent ProgID, or
CLSID) that defines uniquely the Manager responsible for the instantiation of the Primary PMC Object.

#### REQ-MGR-12: A Primary PMC Object created via a Manager implements persistence. {#req-mgr-12}

*<u>Rationale:</u>* As per the design decision, persistence is mandatory for all Primary PMC Objects created via a Manager.

#### REQ-MGR-013: A Manager advertises whether it supports creating a new instance from scratch. {#req-mgr-13}

*<u>Rationale</u>*: A Manager may or may not support creation of a new instance w/o a Template. The PME can properly arrange its user interface (for example enabling/disabling “new instance”) if it knows whether the Manager supports or not this functionality.

#### REQ-MGR-14: PME must initialize the Primary PMC object after creation.

*<u>Rationale</u>*: To conform with the Utilities interface specification (iv), the PME must initialize the Primary PMC Object, irrespective of the way this object was created by the Manager.

### Use Cases

Use Cases defined here are generic Use Cases for Package Managers. Use Cases specific to a particular category of Package Managers are found in the Interface Specification Document where the particular category is defined.

#### Actors

**PME**. A Process Modelling Environment also known as a CAPE-OPEN Simulator Executive (COSE).

**Flowsheet Builder**. Human actor.

#### List of Use Cases

[UC-MGR-01 Enumerate managers](#uc-mgr-01)

[UC-MGR-02 Create and Initialize a Manager](#uc-mgr-02)

[UC-MGR-03 Enumerate Templates Available in a Manager](#uc-mgr-03)

[UC-MGR-04 Select a template](#uc-mgr-04) 

[UC-MGR-05 Check Whether Manager Supports Creation without A Template](#uc-mgr-05)

[UC-MGR-06 Create and Customize a Primary PMC Object Without Using a Template](#uc-mgr-06)

[UC-MGR-07 Create a Primary PMC Object from a Template](#uc-mgr-07)

[UC-MGR-08 Restore a Primary PMC](#uc-mgr-08)

[UC-MGR-09 Configure a Manager](#uc-mgr-09)

#### Use Case map

<img src="media\image3.png" style="width:6.925in;height:5.19722in" alt="Diagram Description automatically generated" />

<span id="_Toc117577707" class="anchor"></span>Figure 2‑1 Use Case map with PME or Flowsheet Builder as Actors

#### Use Case Priorities

**High**. Mandatory functionality for a Manager. Functionality without which the operation usability or performance of a Manager might be seriously compromised. Implementations that do not fulfil high priority Use Cases are not CAPE-OPEN compliant.

**Medium**. Very desirable functionality that will make Managers more usable, transportable, or versatile. The essence of a Manager is not compromised by not supporting this Use Case, although the usability and acceptance of the component can be.

**Low**. Desirable functionality that will improve the performance of Managers. If this Use Case is not met usability or acceptance can decrease.

#### Description of Use Cases

##### UC-MGR-01 Enumerate managers {#uc-mgr-01}

Typically, a PME displays to the Process Engineer a list of available stand-alone Primary PMCs as well as a list of Managers providing Primary PMCs of the same type. Use Case fulfils the second part of this process.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;This Use Case is fulfilled by middleware functionalities&gt;|
|**<u>Pre-conditions:</u>**| &lt;None&gt;|
|**<u>Flow of events:</u>**|From the middleware component registry, PME enumerates all available Managers fitting a given purpose (Managers are registered using the Category ID associated with the Manager type). |
|**<u>Post-conditions:</u>**|&lt;The available Managers have been enumerated&gt;|
|**<u>Errors:</u>**|None|
|**<u>Uses:</u>**|None|

##### UC-MGR-02 Create and Initialize a Manager {#uc-mgr-02}

Use Case applies a generic Use Case, applicable to all Primary PMCs, within the context of the Manager Common interface specification. This Use Case does not introduce any additional feature to the generic Use Case.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;This Use Case is fulfilled by middleware functionalities and *ICapeUtilities::Initialize*.&gt;|
|**<u>Pre-conditions:</u>**| &lt;None&gt;|
|**<u>Flow of events:</u>**|1. PME instantiates the Manager using middleware functionalities.<br>2. PME sets the Simulation Context on the Manager.<br>3. PME initializes the Manager using *ICapeUtilities::Initialize*.<br>Note: at the end of the lifespan of the Manager (at the latest, at the time of closing the PME), the Manager must be terminated by the PME (using *ICapeUtilities::Terminate*).|
|**<u>Post-conditions:</u>**|&lt;The Manager is ready for use.&gt;|
|**<u>Errors:</u>**|&lt;The Manager cannot be created.&gt;|
|**<u>Uses:</u>**|None.|

##### UC-MGR-03 Enumerate Templates Available in a Manager {#uc-mgr-03}

Flowsheet Builder wants to know which Templates are available from the Manager. Therefore, the PME needs the list of Templates.

The PME should not instantiate all available Managers but should instantiate the Manager(s) for which the Flowsheet Builder requests the list of Templates. Instantiation of all Managers on the system, without the Flowsheet Builder explicitly asking for it, may lead to undesired side effects such as checking out and locking licenses, substantial delays, and makes the PME susceptible to unavoidable crashes in case of an implementation error in any Manager on the system.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;Use Case is fulfilled by [*ICapeManager::GetTemplateList*](#icapemanagergettemplatelist) method.&gt;|
|**<u>Pre-conditions:</u>**| &lt;The Manager has been created using [UC-MGR-02](#uc-mgr-02).&gt;|
|**<u>Flow of events:</u>**|1. The PME calls [*GetTemplateList*](#icapemanagergettemplatelist).<br>2. The Manager returns a list of Template names (list could be empty).</li></ol>|
|**<u>Post-conditions:</u>**|&lt;The available Templates have been enumerated and can be presented to the Process Engineer.&gt;|
|**<u>Errors:</u>**|&lt;The Manager fails to provide a list of Template names.&gt;|
|**<u>Uses:</u>**|None|


##### UC-MGR-04 Select a template {#uc-mgr-04}

|<u>Actor</u>:| &lt;Flowsheet Builder&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;Use Case is fulfilled by functionality internal to the PME.&gt;|
|**<u>Pre-conditions:</u>**| &lt;[UC-MGR-03](#uc-mgr-03) has been executed successfully.&gt;|
|**<u>Flow of events:</u>**|1. The PME presents a list of Template names to the Flowsheet Builder.<br>2. The Flowsheet Builder selects a Template name from the list.<br>Note that the Flowsheet Builder is provided only the names of the Templates and no other information. Upon request from the Flowsheet Builder, the PME may offer access to more information on a given Template. Access is obtained by instantiating a Primary PMC Object from the Template (exercising [UC-MGR-07](#uc-mgr-07). It is not advisable for the PME to automatically instantiate a Primary PMC Object for each available Template in the list as this may be a very expensive and error-prone operation.|
|**<u>Post-conditions:</u>**|&lt;A Template has been selected.&gt;|
|**<u>Errors:</u>**|&lt;Flowsheet Builder has cancelled the selection.&gt;|
|**<u>Uses:</u>**|&lt;[UC-MGR-07](#uc-mgr-07)&gt;|

##### UC-MGR-05 Check Whether Manager Supports Creation without A Template {#uc-mgr-05}

Context: the PME wants to provide the Flowsheet Builder with the possibility to create a Primary PMC Object from a Manager without using a Template. In a typical workflow, the PME checks if the Manager supports such a functionality: it allows the PME to enable/disable appropriate controls within its GUI.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|****<u>Status:</u>****| &lt;Use Case is fulfilled through property [*SupportsCreateNew* of *ICapeManager*](#icapemanagersupportscreatenew).&gt;|
|****<u>Pre-conditions:</u>****| &lt;[UC-MGR-02](#uc-mgr-02) has been executed successfully.&gt;|
|****<u>Flow of events:</u>****|The PME obtains the value of the property [*SupportsCreateNew* from *ICapeManager*](#icapemanagersupportscreatenew).|
|****<u>Post-conditions:</u>****|&lt;Value of property [*SupportsCreateNew*](#icapemanagersupportscreatenew) is known to the PME.&gt;<br> &lt;The PME arranges its user interface accordingly.&gt;|
|****<u>Errors:</u>****|None|
|****<u>Uses:</u>****|None|

##### UC-MGR-06 Create and Customize a Primary PMC Object Without Using a Template {#uc-mgr-06}

Context: The Process Engineer wants to create a custom Primary PMC object from scratch. The created Primary PMC Object is for use in the current PME context.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;Medium&gt;|
|**<u>Status:</u>**| &lt;Use Case is fulfilled by [*ICapeManager::CreateNew*](#icapemanagercreatenew) and by property [*ICapeManager::LastCreateNewWasCanceledByUser*](#icapemanagerlastcreatenewwascanceledbyuser).&gt;|
|**<u>Pre-conditions:</u>**| &lt;[UC-MGR-02](#uc-mgr-02) has been executed successfully.&gt;|
|**<u>Flow of events:</u>**|<ol><li>The PME optionally exercises [UC-MGR-05](#uc-mgr-05).<ol type="a"><li>If the Manager does not support creation of a Primary PMC object without using a Template, Use Case stops here.</li></ol><li>The PME exercises [*ICapeManager::CreateNew*](#icapemanagercreatenew).</li><li>The Manager creates a new Primary PMC Object without using any Template. The Primary PMC Object may be a default or an empty Primary PMC Object. The Manager may let the Flowsheet Builder performs customization of the Primary PMC object, using means private to the Manager.<ol type="a"><li>If the Flowsheet Builder cancels the creation, the Manager raises an exception.</li><li>The PME may request the [*ICapeManager::LastCreateNewWasCanceledByUser*](#icapemanagerlastcreatenewwascanceledbyuser) property from the Manager to decide whether to display an error message to the Flowsheet Builder. Use Case stops here.</li></ol><li>The Manager returns a new Primary PMC Object.</li></ol>|
|**<u>Post-conditions:</u>**|&lt;A Primary PMC Object is available to the PME.&gt;|
|**<u>Errors:</u>**|&lt;Flowsheet Builder cancelled the creation.&gt;;<br> &lt;Manager does not support creation of a Primary PMC Object from scratch.&gt;; <br>&lt;The PMC instance cannot be created and is not delivered to the PME (for example out of memory, license issue).&gt;|
|**<u>Uses:</u>**|[UC-MGR-05](#uc-mgr-05)|

##### UC-MGR-07 Create a Primary PMC Object from a Template {#uc-mgr-07}

Context: the Flowsheet Builder may have selected a Template name using [UC-MGR-04](#uc-mgr-04) or a Template name is known to the PME. This Use Case describes the direct instantiation of a Primary PMC object that the PME launches before any use of the object. This is a common and necessary step in all procedures leading to exercising a Primary PMC within a Flowsheet.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;Use Case is fulfilled by *ICapeManager::CreateFromTemplate* and by middleware functionalities.&gt;|
|**<u>Pre-conditions:</u>**|&lt;[UC-MGR-02](#uc-mgr-02) has been executed successfully.&gt;<br>&lt;The PME knows which previously configured PMC Object must be instantiated, for example through execution of [UC-MGR-04](#uc-mgr-04).&gt;|
|**<u>Flow of events:</u>**|The PME executes *ICapeManager::CreateFromTemplate* with the name of the Template as input argument.<br>If the Template name is recognized, the Manager creates an instance of the Primary PMC, and delivers it to the PME.|
|**<u>Post-conditions:</u>**|&lt;An object representing the Primary PMC is instantiated.&gt;|
|**<u>Errors:</u>**|&lt;The Template is not recognised by the Manager.&gt;;<br> &lt;The PMC instance cannot be created and is not delivered to the PME (for example out of memory, license issue).&gt;|
|**<u>Uses:</u>**|None|

##### UC-MGR-08 Restore a Primary PMC {#uc-mgr-08}

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;High&gt;|
|**<u>Status:</u>**| &lt;This Use Case is fulfilled by the method [*CreateForLoad* of *ICapeManager*](#icapemanagercreateforload) interface.&gt;|
|**<u>Pre-conditions:</u>**|&lt;[UC-MGR-02](#uc-mgr-02) has been executed successfully.&gt;<br>&lt;Data that was previously persisted by the Primary PMC Object is available to the PME.&gt;|
|**<u>Flow of events:</u>**|The PME calls [*CreateForLoad*](#icapemanagercreateforload) on the Manager. The Manager returns an un-configured Primary PMC Object.<br>The PME optionally provides the returned Primary PMC Object with a Simulation Context by invoking *ICapeUtilities::SetSimulationContext*.<br>The PME restores the data saved previously by the Primary PMC Object, invoking Load on the persistence interface implemented by the Primary PMC Object.<br>The PME initializes the restored Primary PMC Object using *ICapeUtilities::Initialize*.|
|**<u>Post-conditions:</u>**|&lt;The PMC is available and initialized.&gt;<br>&lt;The PMC specific data have been restored.&gt;|
|**<u>Errors:</u>**|&lt;The PMC instance cannot be created and is not delivered to the PME (for example out of memory, license issue).&gt;|
|**<u>Uses:</u>**|None|

##### UC-MGR-09 Configure a Manager {#uc-mgr-09}

Context: Some Managers may have external means of configuration, for example save a particular document as a Template for a CAPE-OPEN Manager. As such, functionality falls out of the CAPE-OPEN scope and is not described here.

A PME can also request configuration of a Manager using *ICapeUtilities::Edit*. Any change of configuration of a Manager applies to the current machine on which the Manager is installed and should typically not be persisted.

Per design, administration of Templates (e.g., add, remove, rename, edit a Template) is private to the Manager and falls outside of the scope of this interface specification. Priority is medium since not all Managers support editing. However PMEs should allow for any Manager to be configured though *ICapeUtilities::Edit*.

|<u>Actor</u>:| &lt;PME&gt;|
|---|---|
|**<u>Priority:</u>**| &lt;Medium&gt;|
|**<u>Status:</u>**| &lt;Use Case is fulfilled through *ICapeUtilities::Edit*.&gt;|
|**<u>Pre-conditions:</u>**|&lt;[UC-MGR-02](#uc-mgr-02) has been executed successfully.&gt;|
|**<u>Flow of events:</u>**|The PME executes *ICapeUtilities::Edit* on the Manager. The Manager may show a modal window in which it allows the Process Engineer to reconfigure the available Templates.<br>Once Edit is finished and returns OK, the PME assumes that the list of Templates available from the Manager has changed.|
|**<u>Post-conditions:</u>**|&lt;New list of Templates is also available to the PME.&gt;<br>&lt;New list of Templates is also available to other PMEs&gt;|
|**<u>Errors:</u>**|&lt;The Manager does not implement *ICapeUtilities::Edit* (raises a not-implemented error).&gt;|
|**<u>Uses:</u>**|None|

## Analysis and Design

A Manager is a Primary PMC object. A PME must use it as such. The following sequence diagram exemplifies how a new instance of a Manager is used by a PME.

<img src="media\image4.png" style="width:4.28125in;height:6.20833in" alt="Diagram Description automatically generated with low confidence" />

<span id="_Toc117577708" class="anchor"></span>Figure 3‑1 Example of Manager lifespan

The following state diagram describes which are the states through which a Manager goes, and emphasizes the need to terminate the Manager.

<img src="media\image5.png" style="width:3.78125in;height:4.55208in" alt="Diagram Description automatically generated" />

<span id="_Toc117577709" class="anchor"></span>Figure 3‑2 State diagram of Manager

The sequence of actions around the creation of a Primary PMC Object based on a Template available from a Manager is exemplified below:

<img src="media\image6.png" style="width:6.925in;height:5.02292in" alt="A picture containing table Description automatically generated" />

<span id="_Toc117577710" class="anchor"></span>Figure 3‑3 Using a Primary PMC created from a Template

### Overview

A Manager is a Primary PMC component. The Manager implements the *ICapeManager* interface as well as CAPE-OPEN common interfaces mandatory for a Primary Process Modelling Component that itself derives from any CAPE-OPEN Component.

<img src="media\image7.png" style="width:3.84375in;height:1.45833in" alt="Diagram, schematic Description automatically generated" />

<span id="_Ref116487029" class="anchor"></span>Figure 3‑4 Interfaces implemented by any CAPE-OPEN Object

<img src="media\image8.png" style="width:4.8125in;height:2.30208in" alt="Diagram Description automatically generated" />

<span id="_Ref116486972" class="anchor"></span>Figure 3‑5 Interfaces
implemented by any Primary PMC object 

As per the diagrams in Figure 3‑4 and in Figure 3‑5, a Manager must handle its errors through CAPE-OPEN defined means (ix).

The component diagram of a Manager is as shown in Figure 3‑6.

<img src="media\image9.png" style="width:4.45833in;height:2.55208in" alt="Diagram Description automatically generated" />

<span id="_Ref89001138" class="anchor"></span>Figure 3‑6 Component diagram: Manager

The concept of Manager presumes that all Primary PMC objects created by a Manager are persistable Primary PMC objects as depicted in Figure 3‑7.

<img src="media\image10.png" style="width:5.13542in;height:2.70833in" alt="Diagram Description automatically generated" />

<span id="_Ref116487099" class="anchor"></span>Figure 3‑7 Persistable Primary PMC

### Manager

A Manager component is a primary CAPE-OPEN component and is registered as a CAPE-OPEN component. There is no specific Category ID for any Manager.

However, all specific Managers are assigned a Category ID by the specification document that specifies the specific Manager. The implemented category depends on what category of CAPE-OPEN components it is managing. The category IDs are documented in the relevant interface specifications.

Note that a Manager may have more than one type of managed Objects. For example, a Manager, for which all objects are Property Packages supporting Chemical Reactions, may be identified both as a Property Package Manager and as a Chemical Reaction Package Manager. Be aware that all objects exposed by a Manager are expected to implement functionality for all PMC type advertised through the Manager Category ID.

### Interface description

The following figure describes the *ICapeManager* interface.

<img src="media\image11.png" style="width:5.81458in;height:1.9101in" alt="Graphical user interface, text, application, chat or text message Description automatically generated" />

<span id="_Toc117577715" class="anchor"></span>Figure 3‑8 Interface
diagram of *ICapeManager*

#### ICapeManager.SupportsCreateNew 

| Interface Name | *ICapeManager*    |
|----------------|-------------------|
| Property Name  | SupportsCreateNew |
| Access         | Read-only         |
| Returns        | CapeBoolean       |

Description

Returns True if the Manager supports creation of new Primary PMC Objects.

Errors

Notes

If *CreateNew* function is not implemented, the property value is False. The PME can use this property to properly arrange its user interface, for example enable/disable a button.

#### ICapeManager.LastCreateNewWasCanceledByUser 

| Interface Name | *ICapeManager*                 |
|----------------|--------------------------------|
| Property Name  | LastCreateNewWasCanceledByUser |
| Access         | Read-only                      |
| Returns        | CapeBoolean                    |

Description

Returns True if within last execution of *CreateNew* in the current thread, operation was cancelled by Process Engineer.

Errors

Notes

If *CreateNew* was not previously called in the current thread, the property may raise an error or have an arbitrary value.

#### ICapeManager.CreateForLoad 

| Interface Name | *ICapeManager* |
|----------------|----------------|
| Method Name    | CreateForLoad  |
| Returns        | CapeInterface  |

Description

Returns an instance of the Primary Process Modelling Component object that the PME will de-persist.

Arguments

None

Errors

No specific error.

Notes

As per the requirements in the Persistence Common Interface Specification (vi) and the Utilities Common Interface Specification (iv), before using the created Object, the PME must invoke Load on the persistence interface of the created Object, followed by initialization (*ICapeUtilities::Initialize*). Simulation Context may be set at any time, preferably before calling Load.

#### ICapeManager.CreateNew 

| Interface Name | *ICapeManager* |
|----------------|----------------|
| Method Name    | CreateNew      |
| Returns        | CapeInterface  |

Description

Returns a Primary PMC Object that is not based on a Template and is not de-persisted.

Arguments

None

Errors

*ECapeNotImplemented* – The error to be used if the Manager does not allow for the creation of a new PMC Object through the interface. The Manager must not return *ECapeNotImplemented* if *SupportsCreateNew* is TRUE.

Notes

In accordance with *ICapeIdentification* interface specification (x), the returned PMC Object should have a name. The PME is allowed to subsequently change the name, using *ICapeIdentification* interface.

A new instance is not based on a Template and may be a blank, un-configured instance, or a default instance, or the Manager may require the Process Engineer to configure the new instance on the fly. Therefore, the Manager is allowed to show a modal dialog during the process of creating a new instance. This user interface can be the same as the one launched by *ICapeUtilities::Edit* implemented on the PMC Object. The Manager is allowed to fail the creation of a new instance if the Process Engineer does not provide suitable configuration input.

In CAPE-OPEN 1.2 this method will carry a CapeWindowId input argument that describes the parent window for the dialog, to conform to *ICapeUtilities::Edit*.

#### ICapeManager.GetTemplateList 

| Interface Name | *ICapeManager*  |
|----------------|-----------------|
| Method Name    | GetTemplateList |
| Returns        | CapeArrayString |

Description

Returns the list of the names of all Templates available from the Manager.

Arguments

None

Errors

There is no specific error for this method.

Notes

The returned list can be empty.

#### ICapeManager.CreateFromTemplate 

| Interface Name | *ICapeManager*     |
|----------------|--------------------|
| Method Name    | CreateFromTemplate |
| Returns        | CapeInterface      |

Description

Returns a Primary PMC Object based on the Template specified by the client of the Manager.

Arguments

| Name                  | Type       | Description                         |
|-----------------------|------------|-------------------------------------|
| \[in\] *templateName* | CapeString | The name of the requested Template. |

Errors

*ECapeInvalidArgument* – Raised if *templateName* argument does not identify a Template known to the Manager.

Notes

The list of names of available Template can be obtained by executing *GetTemplateList*.

## Scenarios 

This chapter lists possible scenarios for a Manager component.

### Property Package

Property Packages can be managed by a Property Package Manager. 

<img src="media\image12.png" style="width:3.42708in;height:2.88542in" alt="Diagram Description automatically generated" />

<span id="_Toc117577716" class="anchor"></span>Figure 4‑1 Property Package Manager

### Reaction Package

Chemical Reaction Packages are managed by a Chemical Reaction Package Manager.

<img src="media\image13.png" style="width:2.96875in;height:2.5in" alt="Diagram Description automatically generated" />

<span id="_Toc117577717" class="anchor"></span>Figure 4‑2 Chemical Reaction Package Manager

## Prototypes implementation 

None.

## Glossary

Only terms specific to the Chemical Reactions interfaces are defined in this section. For definition of other terms used in this document, refer to the glossary in the Thermodynamic and Physical Properties Interface specification (iii) document in version 1.1.

### Flowsheet Builder

The person who sets up the flowsheet, the structure of the flowsheet, chooses thermodynamic models and the unit operation models that are in the flowsheet. This person hands over a working flowsheet to the Flowsheet User (The person who uses an existing flowsheet. This person will put new data into the flowsheet, rather than change the structure of the flowsheet). The Flowsheet Builder can act as a Flowsheet User.

### Instance

An instance is a Primary Process Modelling Component. It is a software component created by a Manager from a Template.

### Template

A Manager operates on a list of Templates. A Template contains the configuration of the instance to be created by the Manager. Multiple instances can be created by the Manager from a given Template. Once an instance is created from a Template, the configuration of the instance may be modified through functionality provided by the instance. Any configuration change of the instance does not affect the Template.

## Bibliography

CAPE-OPEN Interface Specification: Thermodynamic and Physical Properties Version 1.1

CAPE-OPEN Interface Specification: Chemical Reactions Interface Specification 1.1

CAPE-OPEN Interface Specification: Unit Operation Specification

CAPE-OPEN Interface specification: Utilities Common Interface

CAPE-OPEN Interface Specification: Parameter Common Interfaces

CAPE-OPEN Interface Specification: Persistence Common Interfaces

CAPE-OPEN Methods and Tools integrated guidelines

CAPE-OPEN Interface Specification: Simulation Context COSE Interface

CAPE-OPEN Error handling Interface

CAPE-OPEN Interface Specification: Identification.
