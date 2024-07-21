; SPIR-V
; Version: 1.3
; Generator: Google Shaderc over Glslang; 11
; Bound: 28
; Schema: 0
               OpCapability Shader
               OpCapability MultiView
          %1 = OpExtInstImport "GLSL.std.450"
               OpMemoryModel Logical GLSL450
               OpEntryPoint Fragment %main "main" %FragColor %xy %gl_ViewIndex
               OpExecutionMode %main OriginUpperLeft
               OpSource GLSL 450
               OpSourceExtension "GL_EXT_multiview"
               OpSourceExtension "GL_GOOGLE_cpp_style_line_directive"
               OpSourceExtension "GL_GOOGLE_include_directive"
               OpName %main "main"
               OpName %FragColor "FragColor"
               OpName %xy "xy"
               OpName %gl_ViewIndex "gl_ViewIndex"
               OpDecorate %FragColor Location 0
               OpDecorate %xy Location 0
               OpDecorate %gl_ViewIndex Flat
               OpDecorate %gl_ViewIndex BuiltIn ViewIndex
       %void = OpTypeVoid
          %3 = OpTypeFunction %void
      %float = OpTypeFloat 32
    %v4float = OpTypeVector %float 4
%_ptr_Output_v4float = OpTypePointer Output %v4float
  %FragColor = OpVariable %_ptr_Output_v4float Output
    %v2float = OpTypeVector %float 2
%_ptr_Input_v2float = OpTypePointer Input %v2float
         %xy = OpVariable %_ptr_Input_v2float Input
       %uint = OpTypeInt 32 0
     %uint_0 = OpConstant %uint 0
%_ptr_Input_float = OpTypePointer Input %float
     %uint_1 = OpConstant %uint 1
        %int = OpTypeInt 32 1
%_ptr_Input_int = OpTypePointer Input %int
%gl_ViewIndex = OpVariable %_ptr_Input_int Input
    %float_1 = OpConstant %float 1
       %main = OpFunction %void None %3
          %5 = OpLabel
         %16 = OpAccessChain %_ptr_Input_float %xy %uint_0
         %17 = OpLoad %float %16
         %19 = OpAccessChain %_ptr_Input_float %xy %uint_1
         %20 = OpLoad %float %19
         %24 = OpLoad %int %gl_ViewIndex
         %25 = OpConvertSToF %float %24
         %27 = OpCompositeConstruct %v4float %17 %20 %25 %float_1
               OpStore %FragColor %27
               OpReturn
               OpFunctionEnd
