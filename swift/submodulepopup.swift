struct SubmodulePopupView: View {
    let submodule: ObservableAUParameterGroup
    @State private var showDetails = false

    var body: some View {
        Button("Show Details") {
            showDetails.toggle()
        }
        .popover(isPresented: $showDetails) {
            AUParameterTreeView(node: submodule)
                .frame(width: 300, height: 400)
        }
    }
}
