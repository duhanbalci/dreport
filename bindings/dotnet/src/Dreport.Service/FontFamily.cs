using System.Text.Json.Serialization;

namespace Dreport.Service;

/// <summary>One font family with its registered variants.</summary>
public sealed record FontFamily(
    [property: JsonPropertyName("family")] string Family,
    [property: JsonPropertyName("variants")] IReadOnlyList<FontVariant> Variants);

/// <summary>One weight/italic combination within a family.</summary>
public sealed record FontVariant(
    [property: JsonPropertyName("weight")] ushort Weight,
    [property: JsonPropertyName("italic")] bool Italic);
