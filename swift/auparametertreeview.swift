import SwiftUI
import AudioToolbox

// MARK: - Parameter View

struct ParameterView: View {
    @Bindable var param: ObservableAUParameter

    var body: some View {
        VStack(alignment: .leading) {
            Text(param.displayName)
                .font(.subheadline)
            controlForParameter()
        }
    }

    @ViewBuilder
    private func controlForParameter() -> some View {
        switch param.unit {
        case .boolean:
            Toggle(isOn: Binding(
                get: { param.boolValue },
                set: {
                    param.onEditingChanged(true)
                    param.boolValue = $0
                    param.onEditingChanged(false)
                }
            )) {
                Text("Enabled")
            }

        case .indexed where param.parameter?.valueStrings != nil:
            Picker(param.displayName, selection: $param.value) {
                ForEach(0..<param.parameter!.valueStrings!.count, id: \.self) { index in
                    Text(param.parameter!.valueStrings![index])
                        .tag(Float(index))
                }
            }
            .pickerStyle(MenuPickerStyle())

        default:
            HStack {
                Slider(
                    value: Binding(
                        get: { param.value },
                        set: {
                            param.onEditingChanged(true)
                            param.value = $0
                            param.onEditingChanged(false)
                        }
                    ),
                    in: param.min...param.max
                )
                Text("\(param.value, specifier: "%.2f") \(unitSuffix(for: param.unit))")
                    .font(.caption)
                    .frame(minWidth: 60, alignment: .trailing)
            }
        }
    }

    private func unitSuffix(for unit: AudioUnitParameterUnit) -> String {
        switch unit {
        case .hertz: return "Hz"
        case .decibels: return "dB"
        case .percent: return "%"
        case .milliseconds: return "ms"
        case .seconds: return "s"
        case .cents: return "cents"
        case .relativeSemiTones: return "st"
        case .linearGain: return "x"
        case .degrees: return "°"
        default: return ""
        }
    }
}

// MARK: - Tree View

struct ObservableAUParameterTreeView: View {
    var parameterTree: ObservableAUParameterGroup

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 8) {
                ForEach(parameterTree.children.sorted(by: { $0.key < $1.key }), id: \.key) { _, child in
                    ObservableAUParameterNodeView(node: child)
                }
            }
            .padding()
        }
    }
}

struct ObservableAUParameterNodeView: View {
    let node: ObservableAUParameterNode

    var body: some View {
        if let param = node as? ObservableAUParameter {
            ParameterView(param: param)
        } else if let group = node as? ObservableAUParameterGroup {
            CollapsibleGroup(group: group)
        }
    }
}

// MARK: - Collapsible Group View

struct CollapsibleGroup: View {
    let group: ObservableAUParameterGroup
    @State private var isExpanded: Bool = true

    var body: some View {
        DisclosureGroup(isExpanded: $isExpanded) {
            ForEach(group.children.sorted(by: { $0.key < $1.key }), id: \.key) { _, child in
                ObservableAUParameterNodeView(node: child)
            }
        } label: {
            Text("Group")
                .font(.headline)
        }
    }
}
