// ********************************************************************
//
// Using the SMST Component: Sequential Modular Analysis on a Flowsheet
//
// May 3th 1999 - JPB - CAPE-OPEN Project
//
// ********************************************************************
//
//


// Include Files
#include <conio.h>
#include <fstream.h>
#include "ICapeNumericSMST_c.hh"


// The main program
int main(int argc, char* const* argv) {

  try {

    // Initialize the ORB  -----------------------------------------------------------

    CORBA::ORB_var orb = CORBA::ORB_init(argc, argv);


	// "Bind" Process to a ICapeNumericGATComponent Object using the Trader file  ---------------

	ifstream infile("Trader.txt", ios::nocreate);
	if (!infile) throw "Trader File Error";
	char IOR[527];
	infile >> IOR ;
	infile.close();
	ICapeNumericGATComponent_ptr cocomponent = ICapeNumericGATComponent::_narrow(orb->string_to_object(IOR));

	cout<<endl;
	cout<<"  -->  Bind to a CO Compliant SMST Component "<<endl;
	cout<<"  ----------------------------------------------------------------------------------"<<endl;
	cout<<"    Name        : "<<cocomponent->GetComponentName()<<endl;	
	cout<<"    Version     : "<<cocomponent->GetVersionNumber()<<endl;
	cout<<"    Description : "<<cocomponent->GetComponentDescription()<<endl;
	cout<<"    Location    : ";
	if(cocomponent->_is_remote()==char(1)) cout<<"Remote Object"<<endl;
	else cout<<"No Remote Object"<<endl;
	cout<<"  ----------------------------------------------------------------------------------"<<endl;
	cout<<endl;
	

	// Build the flowsheet --------------------------------------------------------

	//     Create the Flowsheet Factory
	cout<<endl;
	cout<<"  -->  Connection to the FlowsheetFactory"<<endl;
	ICapeNumericFlowsheetFactory_ptr gserver = cocomponent->CreateFlowsheetFactory("Flowsheet Service");
	if(CORBA::is_nil(gserver)==char(1)) throw "CreateFlowsheetFactory() - ObjRef=NIL";


	//     Create the flowsheet
	cout<<endl;
	cout<<"  -->  Flowsheet Creation"<<endl;
	ICapeNumericProcessGraph_ptr graph;
	CapeString gname="M&W Flowsheet";
	graph=ICapeNumericProcessGraph::_narrow(gserver->CreateFlowsheet(PROCESS_GRAPH, gname));
	if(CORBA::is_nil(graph)==char(1)) throw "CreateFlowsheet() - ObjRef=NIL";

	
	//     Complete the flowsheet         
	cout<<endl;
	cout<<"  -->  Flowsheet Completion"<<endl;

	graph->AddStream(13,8,2);
	graph->AddStream(1,1,2);
	graph->AddStream(11,7,6);
	graph->AddStream(3,2,3);
	graph->AddStream(2,3,1);
	graph->AddStream(5,3,4);
	graph->AddStream(4,4,2);
	graph->AddStream(7,5,3);
	graph->AddStream(10,6,7);
	graph->AddStream(12,7,9);
	graph->AddStream(6,3,5);
	graph->AddStream(8,4,5);
	graph->AddStream(9,5,6);

	graph->AddStreamWeight(13,3.);
	graph->AddStreamWeight(1,5.);
	graph->AddStreamWeight(11,4.);
	graph->AddStreamWeight(3,2.);
	graph->AddStreamWeight(2,2.);
	graph->AddStreamWeight(5,5.);
	graph->AddStreamWeight(4,3.);
	graph->AddStreamWeight(7,1.);
	graph->AddStreamWeight(10,5.);
	graph->AddStreamWeight(12,5.);
	graph->AddStreamWeight(6,4.);
	graph->AddStreamWeight(8,5.);
	graph->AddStreamWeight(9,8.);

	// Build the Numeric Analysis ---------------------------------------------------

	//     Create the Analysis Factory
	cout<<endl;
	cout<<"  -->  Connection to the Numeric Server"<<endl;
	ICapeNumericAnalysisFactory_ptr nserver = cocomponent->CreateAnalysisFactory("Analysis Service");
	if(CORBA::is_nil(nserver)==char(1)) throw "ReleaseAnalysisFactory() - ObjRef=NIL";

	cout<<"         Is the Sequential Modular Analysis available ? : ";
	CapeBoolean reply=nserver->SMAnalysisImplementation();
	if(char(1)==reply) cout<<"Yes"<<endl;
	else cout<<"No"<<endl;

	//     Create the Sequential Modular Analysis
	cout<<endl;
	cout<<"  -->  Sequential Modular Analysis Creation"<<endl;
	ICapeNumericAnalysis_ptr num=nserver->CreateNumericAnalysis(SM_ANALYSIS, "SM Analysis");
	if(CORBA::is_nil(num)==char(1)) throw "CreateNumericAnalysis() - ObjRef=NIL";
	cout<<"         Does the Sequential Modular Analysis have specific Parameters ? :";
	if(char(0)==num->ParameterAvailable()) cout<<" No"<<endl;
	else cout<<" Yes"<<endl;

	//     Perform the Sequential Modular Analysis on the flowsheet
	cout<<endl;
	cout<<"  -->  Sequential Modular Analysis Computation "<<endl;
	num->Perform(graph);

	//     Display the Sequential Modular Analysis Results
	cout<<endl;
	cout<<"  -->  Sequential Modular Analysis Results"<<endl;
	long int partcount=(ICapeNumericSMAnalysis::_narrow(num))->GetPartitionCount();
	for(long int i=1;i<=partcount;i++){
		cout<<"          * Partition number "<<i<<endl;
		CapeLongSequence_var listofunit =(ICapeNumericSMAnalysis::_narrow(num))->GetUnitsInPartition(i);
		cout<<"              which contains the units ";
		for (unsigned long int k=0;k<listofunit->length();k++){
			cout<<listofunit[k]<<" ";
		}
		CapeLongSequence_var listoftornstreams =(ICapeNumericSMAnalysis::_narrow(num))->GetTornStreamsInPartition(i);
		cout<<endl;
		if(listoftornstreams->length()!=0){
			cout<<"              with the torn streams ";
			for (unsigned long int kk=0;kk<listoftornstreams->length();kk++) cout<<listoftornstreams[kk]<<" ";
			cout<<endl;
		}

	}

	//getch();
	// Release Objects References and Exit process

	num->Destroy();
	num->_release();
	nserver->Destroy();
	nserver->_release();
	graph->Destroy();
	graph->_release();
	gserver->Destroy();
	gserver->_release();
	//cocomponent->Shutdown(); If you want to shutdown the server
	cocomponent->_release();

  }
	catch(CORBA::Exception& e) {
	cerr << e << endl;
    return(1);
	}
	catch(const char * s) {
	cerr << s << endl;
	return(1);
	}
  return(0);
}
